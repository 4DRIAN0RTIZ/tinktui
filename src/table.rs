//! Convierte el JSON del resultado en una tabla (cabeceras + celdas).

use serde_json::Value;

const MAX_COL_WIDTH: usize = 40;
const MAX_NESTED_LEN: usize = 60;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CellKind {
    Null,
    Bool,
    Number,
    String,
    Nested,
}

pub struct Cell {
    pub text: String,
    pub kind: CellKind,
}

pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<Cell>>,
    pub widths: Vec<u16>,
}

/// Devuelve `None` para escalares: ahí la vista raw ya es suficiente.
pub fn from_json(value: &Value) -> Option<TableData> {
    let (headers, rows): (Vec<String>, Vec<Vec<Cell>>) = match value {
        // Colección de modelos: una fila por elemento, columnas = unión de claves.
        Value::Array(items) if !items.is_empty() && items.iter().all(Value::is_object) => {
            let mut headers: Vec<String> = Vec::new();
            for item in items {
                for key in item.as_object()?.keys() {
                    if !headers.contains(key) {
                        headers.push(key.clone());
                    }
                }
            }
            let rows = items
                .iter()
                .map(|item| {
                    headers
                        .iter()
                        .map(|h| cell(item.get(h).unwrap_or(&Value::Null)))
                        .collect()
                })
                .collect();
            (headers, rows)
        }
        // Lista de escalares (p. ej. `->pluck('email')`).
        Value::Array(items) => {
            let rows = items
                .iter()
                .enumerate()
                .map(|(i, v)| vec![number_cell(i), cell(v)])
                .collect();
            (vec!["#".into(), "valor".into()], rows)
        }
        // Un solo modelo: clave → valor.
        Value::Object(map) => {
            let rows = map
                .iter()
                .map(|(k, v)| {
                    vec![
                        Cell {
                            text: k.clone(),
                            kind: CellKind::String,
                        },
                        cell(v),
                    ]
                })
                .collect();
            (vec!["campo".into(), "valor".into()], rows)
        }
        _ => return None,
    };

    let widths = headers
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let cells = rows.iter().map(|r| r[i].text.chars().count());
            cells
                .chain([h.chars().count()])
                .max()
                .unwrap_or(0)
                .clamp(3, MAX_COL_WIDTH) as u16
        })
        .collect();

    Some(TableData {
        headers,
        rows,
        widths,
    })
}

fn number_cell(n: usize) -> Cell {
    Cell {
        text: n.to_string(),
        kind: CellKind::Number,
    }
}

fn cell(value: &Value) -> Cell {
    match value {
        Value::Null => Cell {
            text: "null".into(),
            kind: CellKind::Null,
        },
        Value::Bool(b) => Cell {
            text: b.to_string(),
            kind: CellKind::Bool,
        },
        Value::Number(n) => Cell {
            text: n.to_string(),
            kind: CellKind::Number,
        },
        Value::String(s) => Cell {
            text: s.replace('\n', "⏎"),
            kind: CellKind::String,
        },
        // Relaciones cargadas con `with()` llegan como objetos/arrays anidados.
        nested => {
            let json = nested.to_string();
            let text = if json.chars().count() > MAX_NESTED_LEN {
                let cut: String = json.chars().take(MAX_NESTED_LEN - 1).collect();
                format!("{cut}…")
            } else {
                json
            };
            Cell {
                text,
                kind: CellKind::Nested,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn coleccion_de_modelos_respeta_orden_y_union_de_columnas() {
        let t =
            from_json(&json!([{ "id": 1, "email": "a@b.c" }, { "id": 2, "extra": true }])).unwrap();
        assert_eq!(t.headers, ["id", "email", "extra"]);
        assert_eq!(t.rows.len(), 2);
        assert!(t.rows[0][2].kind == CellKind::Null);
    }

    #[test]
    fn modelo_unico_es_clave_valor() {
        let t = from_json(&json!({ "id": 1, "username": "ldare" })).unwrap();
        assert_eq!(t.headers, ["campo", "valor"]);
        assert_eq!(t.rows[1][1].text, "ldare");
    }

    #[test]
    fn escalares_no_generan_tabla() {
        assert!(from_json(&json!(35)).is_none());
        assert!(from_json(&json!("hola")).is_none());
    }
}
