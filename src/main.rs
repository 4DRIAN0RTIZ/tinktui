mod app;
mod highlight;
mod history;
mod runner;
mod table;
mod ui;

use std::io::stdout;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::{env, fs};

use anyhow::{Context, Result, bail};
use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{EnterAlternateScreen, enable_raw_mode};

use app::App;
use runner::Target;

const USAGE: &str = "\
tinker-tui: TUI para ejecutar código en Laravel Tinker

Uso:
  tinker-tui [-c|--container NOMBRE]   Tinker dentro de un contenedor Docker
  tinker-tui [-l|--local [DIR]]        Tinker local (DIR por defecto: directorio actual)

Sin argumentos usa $TINKER_TUI_CONTAINER si está definida; si no, modo local.";

fn main() -> Result<()> {
    let target = parse_args()?;
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, App::new(target));
    ratatui::restore();
    result
}

fn parse_args() -> Result<Target> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        None => Ok(match env::var("TINKER_TUI_CONTAINER") {
            Ok(name) if !name.is_empty() => Target::Docker(name),
            _ => Target::Local(env::current_dir()?),
        }),
        Some("-c" | "--container") => {
            let name = args.next().context("falta el nombre del contenedor")?;
            Ok(Target::Docker(name))
        }
        Some("-l" | "--local") => {
            let dir = args
                .next()
                .map(PathBuf::from)
                .unwrap_or(env::current_dir()?);
            Ok(Target::Local(dir))
        }
        Some("-h" | "--help") => {
            println!("{USAGE}");
            std::process::exit(0);
        }
        Some(other) => bail!("argumento desconocido: {other}\n\n{USAGE}"),
    }
}

fn run(terminal: &mut DefaultTerminal, mut app: App) -> Result<()> {
    while !app.quit {
        app.poll_runner();
        terminal.draw(|frame| ui::draw(frame, &mut app))?;

        // Timeout corto para que el spinner avance mientras corre Tinker.
        if event::poll(Duration::from_millis(80))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.on_key(key);
        }

        if app.wants_external_editor {
            app.wants_external_editor = false;
            edit_external(terminal, &mut app)?;
        }
    }
    Ok(())
}

/// Suspende la TUI, abre `$VISUAL`/`$EDITOR` (nvim por defecto) sobre un scratch
/// `.php` y, al cerrar, carga su contenido en el editor embebido.
fn edit_external(terminal: &mut DefaultTerminal, app: &mut App) -> Result<()> {
    let dir = dirs::cache_dir()
        .unwrap_or_else(env::temp_dir)
        .join("tinker-tui");
    fs::create_dir_all(&dir)?;
    let path = dir.join("scratch.php");
    // La etiqueta `<?php` es para que el LSP (intelephense, phpactor) trate el
    // archivo como PHP; el runner la quita antes de mandarlo a Tinker.
    fs::write(&path, format!("<?php\n\n{}\n", app.code()))?;

    let editor = env::var("VISUAL")
        .or_else(|_| env::var("EDITOR"))
        .unwrap_or_else(|_| "nvim".into());
    let mut parts = editor.split_whitespace();
    let program = parts.next().unwrap_or("nvim");

    ratatui::restore();
    let status = Command::new(program).args(parts).arg(&path).status();
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    terminal.clear()?;

    status.with_context(|| format!("no se pudo abrir el editor '{editor}'"))?;
    let content = fs::read_to_string(&path)?;
    let code = content
        .trim_start()
        .strip_prefix("<?php")
        .unwrap_or(&content);
    app.set_code(code.trim());
    Ok(())
}
