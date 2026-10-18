//! Coloreado del dump de PsySH para la vista raw.
//!
//! No es un parser: reconoce por línea los patrones que imprime PsySH
//! (`clave: valor`, strings, números, `null`, clases `App\Models\User {#123`).
//! Usa colores ANSI con nombre para respetar el tema de la terminal.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::runner::is_error_line;

pub fn highlight(text: &str) -> Vec<Line<'static>> {
    text.lines().map(line).collect()
}

fn dim() -> Style {
    Style::new().fg(Color::DarkGray)
}

fn line(l: &str) -> Line<'static> {
    if l.starts_with("[!]") {
        return Line::styled(l.to_string(), dim());
    }
    if is_error_line(l) {
        return Line::styled(
            l.to_string(),
            Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
        );
    }

    let mut spans = Vec::new();
    let trimmed = l.trim_start();
    spans.push(Span::raw(l[..l.len() - trimmed.len()].to_string()));
    let mut rest = trimmed;

    if let Some(r) = rest.strip_prefix("= ") {
        spans.push(Span::styled("= ", dim()));
        rest = r;
    }
    if let Some((marker, key, after)) = split_key(rest) {
        if !marker.is_empty() {
            spans.push(Span::styled(marker.to_string(), dim()));
        }
        spans.push(Span::styled(key.to_string(), Style::new().fg(Color::Blue)));
        spans.push(Span::styled(":", dim()));
        rest = after;
    }
    tokens(rest, &mut spans);
    Line::from(spans)
}

/// Detecta `clave:` al inicio, con el marcador de visibilidad opcional de PsySH
/// (`#` protegido, `-` privado, `+` dinámico). Devuelve (marcador, clave, resto).
fn split_key(s: &str) -> Option<(&str, &str, &str)> {
    let marker_len = usize::from(s.starts_with(['#', '+', '-']));
    let body = &s[marker_len..];
    let key_len = body
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(body.len());
    if key_len == 0 {
        return None;
    }
    let after = body[key_len..].strip_prefix(':')?;
    // `::` es un acceso estático, no una clave.
    if after.starts_with(':') || !(after.is_empty() || after.starts_with(' ')) {
        return None;
    }
    Some((&s[..marker_len], &body[..key_len], after))
}

fn tokens(s: &str, spans: &mut Vec<Span<'static>>) {
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let start = i;
        let style = if c == '"' {
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' {
                    i += 1;
                }
                i += 1;
            }
            i = (i + 1).min(chars.len());
            Style::new().fg(Color::Green)
        } else if c.is_ascii_digit()
            || (c == '-' && chars.get(i + 1).is_some_and(char::is_ascii_digit))
        {
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            Style::new().fg(Color::Magenta)
        } else if c == '{' && chars.get(i + 1) == Some(&'#') {
            // Id de objeto de PsySH: `{#9316`.
            i += 2;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            dim()
        } else if c.is_alphabetic() || c == '_' || c == '\\' {
            while i < chars.len()
                && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '\\')
            {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            match word.as_str() {
                "null" | "true" | "false" => Style::new().fg(Color::Yellow),
                _ if word.contains('\\') || word.starts_with(char::is_uppercase) => {
                    Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                }
                _ => Style::default(),
            }
        } else if c.is_whitespace() {
            i += 1;
            Style::default()
        } else {
            i += 1;
            dim()
        };
        spans.push(Span::styled(
            chars[start..i].iter().collect::<String>(),
            style,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconoce_claves_con_marcador() {
        assert_eq!(
            split_key("#password: \"x\""),
            Some(("#", "password", " \"x\""))
        );
        assert_eq!(split_key("id: 18,"), Some(("", "id", " 18,")));
        assert_eq!(split_key("User::count()"), None);
        assert_eq!(split_key("all: ["), Some(("", "all", " [")));
    }
}
