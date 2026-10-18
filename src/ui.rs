//! Render de la interfaz: historial a la izquierda, editor y resultado a la derecha.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Cell, List, ListItem, Paragraph, Row, Table, Wrap};

use crate::app::{App, Focus, Status, View};
use crate::highlight::highlight;
use crate::table::{CellKind, TableData};

const COLUMN_SPACING: u16 = 2;
const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn draw(frame: &mut Frame, app: &mut App) {
    let [main, footer] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(28), Constraint::Min(0)]).areas(main);
    let [editor, output] =
        Layout::vertical([Constraint::Percentage(45), Constraint::Min(0)]).areas(right);

    draw_history(frame, app, left);
    draw_editor(frame, app, editor);
    draw_output(frame, app, output);
    draw_footer(frame, app, footer);
}

fn block(title: impl Into<Line<'static>>, focused: bool) -> Block<'static> {
    let color = if focused {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(color))
        .title(title)
}

fn draw_history(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .history
        .entries()
        .iter()
        .map(|code| {
            let mut lines = code.lines();
            let first = lines.next().unwrap_or_default().to_string();
            let more = if lines.next().is_some() { " …" } else { "" };
            ListItem::new(format!("{first}{more}"))
        })
        .collect();

    let list = List::new(items)
        .block(block(" Historial ", app.focus == Focus::History))
        .highlight_symbol("▶ ")
        .highlight_style(Style::new().add_modifier(Modifier::REVERSED));
    frame.render_stateful_widget(list, area, &mut app.history_state);
}

fn draw_editor(frame: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::Editor;
    let title = format!(" Código PHP · {} ", app.target.label());
    app.editor.set_block(block(title, focused));
    app.editor
        .set_line_number_style(Style::new().fg(Color::DarkGray));
    app.editor.set_cursor_line_style(Style::default());
    app.editor
        .set_placeholder_text("User::latest()->take(5)->get()");
    // Sin foco, el cursor se oculta para no confundir dónde va lo que se escribe.
    let cursor = if focused {
        Style::new().add_modifier(Modifier::REVERSED)
    } else {
        Style::default()
    };
    app.editor.set_cursor_style(cursor);
    frame.render_widget(&app.editor, area);
}

fn draw_output(frame: &mut Frame, app: &mut App, area: Rect) {
    let status = match &app.status {
        Status::Idle => Span::raw(""),
        Status::Running(start) => {
            let i = (start.elapsed().as_millis() / 80) as usize % SPINNER.len();
            Span::styled(
                format!(
                    "{} ejecutando {:.1}s ",
                    SPINNER[i],
                    start.elapsed().as_secs_f32()
                ),
                Style::new().fg(Color::Cyan),
            )
        }
        Status::Done {
            success: true,
            elapsed,
        } => Span::styled(
            format!("✓ {:.2}s ", elapsed.as_secs_f32()),
            Style::new().fg(Color::Green),
        ),
        Status::Done {
            success: false,
            elapsed,
        } => Span::styled(
            format!("✗ {:.2}s ", elapsed.as_secs_f32()),
            Style::new().fg(Color::Red),
        ),
    };

    // Ancho útil para columnas: bordes (2) + símbolo de selección "▶ " (2).
    let table_width = area.width.saturating_sub(4);
    let visible = app.table.as_ref().map_or(0, |t| {
        visible_columns(&t.widths, app.col_offset, table_width)
    });

    let mut title = vec![Span::raw(" Resultado "), status];
    if let Some(table) = &app.table {
        let view = match app.view {
            View::Table => "tabla",
            View::Raw => "raw",
        };
        let total = table.headers.len();
        let cols = if visible < total {
            format!(
                "cols {}–{}/{total}",
                app.col_offset + 1,
                app.col_offset + visible
            )
        } else {
            format!("{total} cols")
        };
        title.push(Span::styled(
            format!("· {} filas · {cols} · {view} (t) ", table.rows.len()),
            Style::new().fg(Color::DarkGray),
        ));
    }
    let block = block(Line::from(title), app.focus == Focus::Output);

    match (&app.table, app.view) {
        (Some(table), View::Table) => {
            let selected = app.table_state.selected();
            let widget = table_widget(table, app.col_offset, visible, selected).block(block);
            frame.render_stateful_widget(widget, area, &mut app.table_state);
        }
        _ => {
            let body = if app.output.is_empty() && matches!(app.status, Status::Idle) {
                Paragraph::new("Ctrl+R o F5 para ejecutar").style(Style::new().fg(Color::DarkGray))
            } else {
                Paragraph::new(highlight(&app.output))
            };
            let body = body
                .block(block)
                .wrap(Wrap { trim: false })
                .scroll((app.output_scroll, 0));
            frame.render_widget(body, area);
        }
    }
}

/// Cuántas columnas caben completas desde `offset`. Si la suma de los
/// `Constraint::Length` excede el ancho, ratatui encoge todas proporcionalmente;
/// por eso se cortan las que sobran en vez de dárselas todas al layout.
fn visible_columns(widths: &[u16], offset: usize, available: u16) -> usize {
    let mut used = 0u16;
    let mut count = 0;
    for &w in &widths[offset..] {
        let need = if count == 0 { w } else { w + COLUMN_SPACING };
        if count > 0 && used.saturating_add(need) > available {
            break;
        }
        used = used.saturating_add(need);
        count += 1;
    }
    // Al menos una aunque sea más ancha que el panel (ratatui la recorta),
    // salvo que no haya columnas (p. ej. `[{}]`).
    count.max(1).min(widths.len().saturating_sub(offset))
}

fn table_widget(
    table: &TableData,
    offset: usize,
    visible: usize,
    selected: Option<usize>,
) -> Table<'_> {
    let cols = offset..offset + visible;
    let header = Row::new(table.headers[cols.clone()].iter().map(|h| {
        Cell::from(h.as_str()).style(Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD))
    }));

    let rows = table.rows.iter().enumerate().map(|(i, row)| {
        let is_selected = selected == Some(i);
        Row::new(row[cols.clone()].iter().map(move |c| {
            // En la fila seleccionada no se colorea por tipo: REVERSED invertiría
            // cada color por separado y la fila se vería a parches.
            let style = match c.kind {
                _ if is_selected => Style::default(),
                CellKind::Null => Style::new()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::ITALIC),
                CellKind::Bool => Style::new().fg(Color::Yellow),
                CellKind::Number => Style::new().fg(Color::Magenta),
                CellKind::String => Style::default(),
                CellKind::Nested => Style::new().fg(Color::Cyan),
            };
            Cell::from(c.text.as_str()).style(style)
        }))
    });

    let widths = table.widths[cols.clone()]
        .iter()
        .map(|&w| Constraint::Length(w));
    Table::new(rows, widths)
        .header(header)
        .column_spacing(COLUMN_SPACING)
        .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol("▶ ")
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let keys = match app.focus {
        Focus::Editor => {
            "Esc salir del editor · Ctrl+R/F5 ejecutar · Ctrl+E abrir $EDITOR · Ctrl+L limpiar · Ctrl+C salir"
        }
        Focus::History => {
            "j/k mover · Enter cargar · d borrar · i editor · Tab resultado · Ctrl+R ejecutar · q salir"
        }
        Focus::Output if app.view == View::Table && app.table.is_some() => {
            "j/k fila · h/l columna · 0/$ primera/última col · g/G inicio/fin · t raw · i editor · Tab historial · q salir"
        }
        Focus::Output if app.table.is_some() => {
            "j/k scroll · d/u página · g/G inicio/fin · t tabla · i editor · Tab historial · q salir"
        }
        Focus::Output => {
            "j/k scroll · d/u página · g/G inicio/fin · i editor · Tab historial · q salir"
        }
    };
    frame.render_widget(
        Paragraph::new(keys).style(Style::new().fg(Color::DarkGray)),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuenta_columnas_que_caben_completas() {
        // 10 + (2+10) = 22 cabe en 25; la tercera (2+10) ya no.
        assert_eq!(visible_columns(&[10, 10, 10], 0, 25), 2);
        assert_eq!(visible_columns(&[10, 10, 10], 1, 25), 2);
        assert_eq!(visible_columns(&[10, 10, 10], 0, 100), 3);
    }

    #[test]
    fn siempre_muestra_una_columna_salvo_que_no_haya() {
        assert_eq!(visible_columns(&[40], 0, 10), 1);
        assert_eq!(visible_columns(&[], 0, 10), 0);
    }
}
