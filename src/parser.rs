pub struct Parser;

impl Parser {
    pub fn cell_name_to_coord(s: &str) -> Option<(u16, u16)> {
        let trimmed = s.trim();
        // Ensure the string starts with a letter.
        let mut chars = trimmed.chars();
        if let Some(first) = chars.next() {
            if !first.is_alphabetic() {
                return None;
            }
        } else {
            return None;
        }
        // Now parse the letter(s) and number(s)
        let letters: String = trimmed.chars().take_while(|c| c.is_alphabetic()).collect();
        let numbers: String = trimmed.chars().skip_while(|c| c.is_alphabetic()).collect();

        if letters.is_empty() || numbers.is_empty() {
            return None;
        }

        // Convert letters to a column index and numbers to a row index.
        let col = letters.chars().fold(0, |acc, c| {
            acc * 26 + ((c.to_ascii_uppercase() as u16) - ('A' as u16) + 1)
        });
        let row = numbers.parse::<u16>().ok()?;

        Some((col, row))
    }

    /// Parses B1+C2 or 2+3 or B1+2 or 2+B1 into (op_code, lhs_str, rhs_str).
    /// op_code: 1='+', 2='-', 3='*', 5='/'.
    pub fn split_binary(expr: &str) -> Option<(char, &str, &str)> {
        // Try each operator in order of precedence or as needed.
        for op in ['+', '-', '*', '/'] {
            if let Some(idx) = expr.find(op) {
                let (lhs, rest) = expr.split_at(idx);
                let rhs = &rest[1..];
                if !lhs.trim().is_empty() && !rhs.trim().is_empty() {
                    return Some((op, lhs, rhs));
                }
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
