//! Ejecuta código en Laravel Tinker (one-shot) sin bloquear la UI.
//!
//! El código se manda por stdin y no con `--execute`: en modo stdin PsySH
//! evalúa todo como un bloque e imprime el valor de retorno de la última
//! sentencia (`= 12`), mientras que `--execute` lo descarta.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;

#[derive(Clone, Debug)]
pub enum Target {
    /// `docker exec -i <contenedor> php artisan tinker`
    Docker(String),
    /// `php artisan tinker` en un directorio local
    Local(PathBuf),
}

impl Target {
    pub fn label(&self) -> String {
        match self {
            Target::Docker(name) => format!("docker:{name}"),
            Target::Local(dir) => format!("local:{}", dir.display()),
        }
    }

    fn command(&self) -> Command {
        let mut cmd = match self {
            Target::Docker(name) => {
                let mut c = Command::new("docker");
                c.args(["exec", "-i", name, "php"]);
                c
            }
            Target::Local(dir) => {
                let mut c = Command::new("php");
                c.current_dir(dir);
                c
            }
        };
        cmd.args(["artisan", "tinker", "--no-ansi"]);
        cmd
    }
}

/// Prefijo de la línea donde el código instrumentado emite el resultado en JSON.
const MARKER: &str = "__TINKER_TUI__";

pub struct RunResult {
    pub output: String,
    /// Valor de la última sentencia serializado con `json_encode` (respeta `$hidden`).
    pub data: Option<Value>,
    pub success: bool,
    pub elapsed: Duration,
}

/// Lanza la ejecución en un hilo y devuelve el canal donde llegará el resultado.
pub fn spawn(target: &Target, code: &str) -> Receiver<RunResult> {
    let (tx, rx) = mpsc::channel();
    let mut cmd = target.command();
    let code = instrument(&prepare_code(code));

    thread::spawn(move || {
        let start = Instant::now();
        let result = run(&mut cmd, &code);
        let elapsed = start.elapsed();
        let msg = match result {
            Ok((output, success)) => {
                let (output, data) = extract_data(&output);
                RunResult {
                    output,
                    data,
                    success,
                    elapsed,
                }
            }
            Err(err) => RunResult {
                output: format!("No se pudo ejecutar Tinker: {err}"),
                data: None,
                success: false,
                elapsed,
            },
        };
        let _ = tx.send(msg);
    });

    rx
}

/// Reescribe la última sentencia para capturar su valor y emitirlo como JSON,
/// sin perder el dump de PsySH:
///
/// ```php
/// $__tui = <última sentencia>;
/// echo "\n__TINKER_TUI__" . json_encode($__tui) . "\n";
/// $__tui;   // PsySH sigue imprimiendo "= ..."
/// ```
///
/// Si la última sentencia no es una expresión (echo, if, foreach...), el código
/// se manda tal cual y solo habrá vista raw.
fn instrument(code: &str) -> String {
    let Some((prefix, last)) = split_last_statement(code) else {
        return code.to_string();
    };
    if !is_expression(last) {
        return code.to_string();
    }
    format!(
        "{prefix}\n$__tui = {last};\n\
         echo \"\\n{MARKER}\" . json_encode($__tui, JSON_PARTIAL_OUTPUT_ON_ERROR | JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES) . \"\\n\";\n\
         $__tui;\n"
    )
}

/// Separa el código en (todo lo anterior, última sentencia), ignorando `;` y `}`
/// dentro de strings, comentarios o paréntesis. Espera que el código termine en `;`.
fn split_last_statement(code: &str) -> Option<(&str, &str)> {
    let body = code.trim_end().strip_suffix(';')?;
    let mut depth = 0i32;
    let mut start = 0;
    let mut chars = body.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        let next = chars.peek().map(|&(_, n)| n);
        match c {
            '\'' | '"' => {
                // Avanza hasta la comilla de cierre respetando escapes.
                while let Some((_, s)) = chars.next() {
                    if s == '\\' {
                        chars.next();
                    } else if s == c {
                        break;
                    }
                }
            }
            '/' if next == Some('/') => skip_line(&mut chars),
            '#' if next != Some('[') => skip_line(&mut chars),
            '/' if next == Some('*') => {
                chars.next();
                let mut prev = ' ';
                for (_, s) in chars.by_ref() {
                    if prev == '*' && s == '/' {
                        break;
                    }
                    prev = s;
                }
            }
            '(' | '[' | '{' => depth += 1,
            ')' | ']' => depth -= 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    start = i + 1;
                }
            }
            ';' if depth == 0 => start = i + 1,
            _ => {}
        }
    }

    if depth != 0 {
        return None;
    }
    let last = body[start..].trim();
    (!last.is_empty()).then(|| (&body[..start], last))
}

fn skip_line(chars: &mut std::iter::Peekable<std::str::CharIndices>) {
    for (_, c) in chars.by_ref() {
        if c == '\n' {
            break;
        }
    }
}

/// Descarta sentencias que no se pueden asignar a una variable.
fn is_expression(stmt: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "echo",
        "print",
        "return",
        "if",
        "for",
        "foreach",
        "while",
        "do",
        "switch",
        "match",
        "use",
        "function",
        "class",
        "interface",
        "trait",
        "enum",
        "namespace",
        "throw",
        "unset",
        "global",
        "static",
        "try",
        "const",
        "abstract",
        "final",
        "declare",
        "break",
        "continue",
        "include",
        "include_once",
        "require",
        "require_once",
        "dd",
    ];
    let word: String = stmt
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    !stmt.starts_with("$__tui") && !KEYWORDS.contains(&word.to_ascii_lowercase().as_str())
}

/// Saca la línea del marcador de la salida y parsea su JSON.
fn extract_data(text: &str) -> (String, Option<Value>) {
    for (pos, _) in text.match_indices(MARKER) {
        let rest = &text[pos + MARKER.len()..];
        let end = rest.find('\n').unwrap_or(rest.len());
        let Ok(data) = serde_json::from_str(&rest[..end]) else {
            continue;
        };

        // PsySH repite el código enviado por stdin, incluido el marcador literal.
        // Solo el marcador cuyo sufijo es JSON corresponde al resultado instrumentado.
        let before = &text[..pos];
        let before = before.strip_suffix('\n').unwrap_or(before);
        let after = &rest[end..];
        let after = after.strip_prefix('\n').unwrap_or(after);
        return (
            format!("{before}{after}").trim_matches('\n').to_string(),
            Some(data),
        );
    }

    (text.to_string(), None)
}

fn run(cmd: &mut Command, code: &str) -> std::io::Result<(String, bool)> {
    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Se escribe y se cierra stdin (drop) para que PsySH sepa que terminó la entrada.
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(code.as_bytes())?;
    }

    let out = child.wait_with_output()?;
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr);
    if !stderr.trim().is_empty() {
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&stderr);
    }

    let text = strip_ansi(&text);
    // PsySH es "exitoso" aunque el código lance una excepción; se detecta por el
    // encabezado que imprime (` Exception `, ` PARSE ERROR `, etc.).
    let has_error = text.lines().any(is_error_line);
    Ok((
        text.trim_matches('\n').to_string(),
        out.status.success() && !has_error,
    ))
}

/// En modo stdin todo es un único bloque PHP, así que cada sentencia necesita `;`.
/// Para no obligar a escribirlo en consultas de una línea, se agrega a la última
/// si falta.
fn prepare_code(code: &str) -> String {
    let mut code = code.trim_end().to_string();
    if let Some(stripped) = code.strip_prefix("<?php") {
        code = stripped.trim_start().to_string();
    }
    if !code.is_empty() && !code.ends_with([';', '}']) {
        code.push(';');
    }
    code.push('\n');
    code
}

pub fn is_error_line(line: &str) -> bool {
    let line = line.trim_start();
    let Some(head) = line.split("  ").next() else {
        return false;
    };
    head == "PARSE ERROR" || head.ends_with("Exception") || head.ends_with("Error")
}

/// Quita secuencias de escape ANSI residuales (por si algo ignora `--no-ansi`).
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                // Parámetros y finalizador de una secuencia CSI (finaliza en 0x40..=0x7E).
                for c in chars.by_ref() {
                    if ('\u{40}'..='\u{7e}').contains(&c) {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agrega_punto_y_coma_si_falta() {
        assert_eq!(prepare_code("User::count()"), "User::count();\n");
        assert_eq!(prepare_code("User::count();  \n"), "User::count();\n");
        assert_eq!(prepare_code("if (true) { 1; }"), "if (true) { 1; }\n");
    }

    #[test]
    fn quita_etiqueta_php() {
        assert_eq!(prepare_code("<?php\n\n1 + 1"), "1 + 1;\n");
    }

    #[test]
    fn detecta_errores_de_psysh() {
        assert!(is_error_line("   Exception  boom."));
        assert!(is_error_line("   PARSE ERROR  PHP Parse error: ..."));
        assert!(is_error_line("   TypeError  Argument ..."));
        assert!(!is_error_line("= 12"));
        assert!(!is_error_line("hola"));
    }

    #[test]
    fn instrumenta_la_ultima_sentencia() {
        let code = instrument("$x = 1;\nUser::count();\n");
        assert!(code.starts_with("$x = 1;\n$__tui = User::count();"));
        assert!(code.contains(MARKER));
        assert!(code.trim_end().ends_with("$__tui;"));
    }

    #[test]
    fn ignora_puntos_y_coma_en_strings_y_closures() {
        let code = "collect([1])->map(function ($n) { return \"a;b\"; });";
        assert_eq!(
            split_last_statement(code),
            Some(("", code.trim_end_matches(';')))
        );
    }

    #[test]
    fn no_instrumenta_sentencias_que_no_son_expresiones() {
        assert_eq!(instrument("echo 'hola';\n"), "echo 'hola';\n");
        assert_eq!(
            instrument("foreach ($a as $b) { dump($b); }\n"),
            "foreach ($a as $b) { dump($b); }\n"
        );
    }

    #[test]
    fn extrae_el_json_de_la_salida() {
        let out = "[!] Aliasing\n\n__TINKER_TUI__[{\"id\":1}]\n= Collection {}";
        let (text, data) = extract_data(out);
        assert_eq!(text, "[!] Aliasing\n= Collection {}");
        assert_eq!(data, Some(serde_json::json!([{ "id": 1 }])));
    }

    #[test]
    fn ignora_el_marcador_repetido_por_psysh() {
        let out = concat!(
            "> $__tui = DB::table('users')->get();\n",
            ". echo \"\\n__TINKER_TUI__\" . json_encode($__tui) . \"\\n\";\n",
            "__TINKER_TUI__[{\"id\":1}]\n",
            "= Illuminate\\Support\\Collection {}"
        );
        let (text, data) = extract_data(out);
        assert_eq!(
            text,
            "> $__tui = DB::table('users')->get();\n. echo \"\\n__TINKER_TUI__\" . json_encode($__tui) . \"\\n\";= Illuminate\\Support\\Collection {}"
        );
        assert_eq!(data, Some(serde_json::json!([{ "id": 1 }])));
    }

    #[test]
    fn limpia_ansi() {
        assert_eq!(strip_ansi("\u{1b}[32m= 12\u{1b}[39m"), "= 12");
    }
}
