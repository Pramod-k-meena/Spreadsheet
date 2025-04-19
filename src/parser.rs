use std::str::FromStr;

/// Parses either a 1‑based cell reference "A1" → Some((col, row)), or returns None.
pub struct Parser;
#[derive(Debug, Clone)]
pub enum Operand {
    Cell((u16, u16)),
    Const(i32),
}

impl Parser {
    pub fn cell_name_to_coord(s: &str) -> Option<(u16, u16)> {
        let mut col = 0u16;
        let mut row_str = String::new();
        for ch in s.chars() {
            if ch.is_ascii_alphabetic() {
                col = col
                    .checked_mul(26)?
                    .checked_add((ch.to_ascii_uppercase() as u16 - b'A' as u16 + 1))?;
            } else if ch.is_ascii_digit() {
                row_str.push(ch);
            } else {
                return None;
            }
        }
        let row = u16::from_str(&row_str).ok()?;
        Some((col, row))
    }

    /// Parses B1+C2 or 2+3 or B1+2 or 2+B1 into (op_code, lhs_str, rhs_str).
    /// op_code: 1='+', 2='-', 3='*', 5='/'.
    pub fn split_binary(expr: &str) -> Option<(i8, &str, &str)> {
        let ops = ['+', '-', '*', '/'];
        for (i, ch) in expr.char_indices().skip(1) {
            if ops.contains(&ch) {
                let code = match ch {
                    '+' => 1,
                    '-' => 2,
                    '*' => 3,
                    '/' => 5,
                    _ => unreachable!(),
                };
                let lhs = expr[..i].trim();
                let rhs = expr[i + 1..].trim();
                return Some((code, lhs, rhs));
            }
        }
        None
    }

    /// Parses MAX(A1:B3), etc., returning (func, start, end).
    pub fn parse_range(expr: &str) -> Option<(&str, (u16, u16), (u16, u16))> {
        let expr = expr.trim();
        for &func in &["MIN", "MAX", "AVG", "SUM", "STDEV", "SLEEP"] {
            let open = format!("{}(", func);
            if expr.starts_with(&open) && expr.ends_with(')') {
                let inside = &expr[open.len()..expr.len() - 1];
                if let Some(colon) = inside.find(':') {
                    let a = inside[..colon].trim();
                    let b = inside[colon + 1..].trim();
                    if let (Some(s), Some(e)) =
                        (Parser::cell_name_to_coord(a), Parser::cell_name_to_coord(b))
                    {
                        return Some((func, s, e));
                    }
                }
            }
        }
        None
    }
}
