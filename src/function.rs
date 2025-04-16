use crate::spreadsheet::{parse_cell_name, Spreadsheet};
use std::iter::Peekable;
use std::str::Chars;
pub const ERR_VALUE: i32 = -999_999;

/// Evaluate an arithmetic expression.
/// This simple parser supports at most one binary operator at the top level.
pub fn evaluate_expression(sheet: &Spreadsheet, expr: &str, which_message: &mut i32) -> i32 {
    let mut chars = expr.chars().peekable();
    let mut result = evaluate_term(sheet, &mut chars, which_message);
    if result == ERR_VALUE {
        return ERR_VALUE;
    }
    let mut binary_count = 0;
    while let Some(&op) = chars.peek() {
        if op == '+' || op == '-' {
            chars.next();
            binary_count += 1;
            if binary_count > 1 {
                *which_message = 1;
                return ERR_VALUE;
            }
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }
            let term = evaluate_term(sheet, &mut chars, which_message);
            if term == ERR_VALUE {
                return ERR_VALUE;
            }
            if op == '+' {
                result += term;
            } else {
                result -= term;
            }
        } else {
            break;
        }
    }
    if chars.peek().is_some() {
        *which_message = 1;
        return ERR_VALUE;
    }
    result
}

/// Evaluate a term (multiplication or division).
pub fn evaluate_term(
    sheet: &Spreadsheet,
    chars: &mut Peekable<Chars>,
    which_message: &mut i32,
) -> i32 {
    let mut result = evaluate_factor(sheet, chars, which_message);
    if result == ERR_VALUE {
        return ERR_VALUE;
    }
    let mut mult_count = 0;
    while let Some(&op) = chars.peek() {
        if op == '*' || op == '/' {
            chars.next();
            mult_count += 1;
            if mult_count > 1 {
                *which_message = 1;
                return ERR_VALUE;
            }
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }
            let factor = evaluate_factor(sheet, chars, which_message);
            if factor == ERR_VALUE {
                return ERR_VALUE;
            }
            if op == '*' {
                result *= factor;
            } else {
                if factor == 0 {
                    *which_message = 5;
                    return ERR_VALUE;
                }
                result /= factor;
            }
        } else {
            break;
        }
    }
    result
}

/// Evaluate a factor: either a numeric literal or a cell reference.
pub fn evaluate_factor(
    sheet: &Spreadsheet,
    chars: &mut Peekable<Chars>,
    which_message: &mut i32,
) -> i32 {
    let mut sign = 1;
    while let Some(&ch) = chars.peek() {
        if ch == '+' || ch == '-' {
            if ch == '-' {
                sign = -sign;
            }
            chars.next();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }
        } else {
            break;
        }
    }
    if let Some(&ch) = chars.peek() {
        if ch.is_ascii_digit() {
            let mut val = 0;
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() {
                    val = val * 10 + d.to_digit(10).unwrap() as i32;
                    chars.next();
                } else {
                    break;
                }
            }
            return sign * val;
        } else if ch.is_ascii_alphabetic() {
            let mut token = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_alphanumeric() {
                    token.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            return sign * get_cell_value(sheet, &token, which_message);
        } else {
            *which_message = 1;
            return ERR_VALUE;
        }
    }
    *which_message = 1;
    ERR_VALUE
}

/// Look up the value of a cell by its name.
pub fn get_cell_value(sheet: &Spreadsheet, cell_name: &str, which_message: &mut i32) -> i32 {
    if let Some((row, col)) = parse_cell_name(cell_name) {
        if row < sheet.rows && col < sheet.cols {
            let val = sheet.cells[sheet.index(row, col)].value;
            if val == ERR_VALUE {
                *which_message = 6;
            }
            return val;
        }
    }
    *which_message = 1;
    ERR_VALUE
}

/// Update a cell’s value from its stored formula (used when recalculating dependents).
pub fn set_only_value(sheet: &mut Spreadsheet, cell_idx: usize) {
    if let Some(formula) = sheet.cells[cell_idx].formula.clone() {
        let mut signal = 0;
        let val = evaluate_expression(sheet, &formula, &mut signal);
        sheet.cells[cell_idx].value = val;
    }
}

/// Set a cell’s formula and value.
/// This function computes the cell index first to avoid overlapping borrows.
pub fn set_cell(
    sheet: &mut Spreadsheet,
    row: usize,
    col: usize,
    expression: &str,
    which_message: &mut i32,
) {
    let cell_idx = sheet.index(row, col);
    crate::dependencies::remove_all_precedents(sheet, cell_idx);
    // First, update the cell's formula in a separate block so that the mutable borrow ends.
    {
        let cell = &mut sheet.cells[cell_idx];
        cell.formula = Some(expression.to_string());
    }
    // Now, no mutable borrow exists; we can safely perform an immutable borrow in evaluate_expression.
    let new_value = evaluate_expression(sheet, expression, which_message);
    // Obtain a new mutable borrow to update the cell's value.
    {
        let cell = &mut sheet.cells[cell_idx];
        cell.value = new_value;
    }
    crate::dependencies::recalc_dependents(sheet, cell_idx);
}
