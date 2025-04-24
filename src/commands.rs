use crate::parser::Parser;
use crate::spreadsheet::Spreadsheet;
use std::io::{self, BufRead, Write};
use std::time::Instant;

pub fn handle_commands(sheet: &mut Spreadsheet) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut input = String::new();
    let mut output_enabled = true;
    let mut viewport_row: usize = 0;
    let mut viewport_col: usize = 0;
    let status_messages = [
        "ok",
        "Invalid cell",
        "Invalid range",
        "unrecognized cmd",
        "Circular dependency",
        "Division_by_zero",
    ];
    let mut which_message: u8 = 0;
    let mut last_instant = Instant::now();

    sheet.display(viewport_row, viewport_col, 10, 10);
    loop {
        let elapsed = last_instant.elapsed().as_secs_f64();
        print!(
            "[{:.1}] ({}) > ",
            elapsed, status_messages[which_message as usize]
        );
        stdout.flush().unwrap();
        input.clear();
        if stdin.lock().read_line(&mut input).unwrap() == 0 {
            break;
        }
        last_instant = Instant::now();
        which_message = 0;
        let input_trimmed = input.trim_end();

        if input_trimmed == "q" {
            break;
        }if input_trimmed == "Q" {
            break;
        } else if input_trimmed == "disable_output" {
            output_enabled = false;
            continue;
        } else if input_trimmed == "enable_output" {
            output_enabled = true;
        } else if input_trimmed.starts_with("scroll_to") {
            let parts: Vec<&str> = input_trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                // Use the parser for label conversion.
                if let Some((col, row)) = Parser::cell_name_to_coord(parts[1]) {
                    // Convert 1-indexed to zero-indexed.
                    let row_idx = row as usize - 1;
                    let col_idx = col as usize - 1;
                    if row_idx < sheet.rows && col_idx < sheet.cols {
                        viewport_row = row_idx;
                        viewport_col = col_idx;
                    } else {
                        which_message = 1;
                    }
                } else {
                    which_message = 1;
                }
            }
        } else if input_trimmed == "w" {
            viewport_row = viewport_row.saturating_sub(10);
        } else if input_trimmed == "s" {
            if sheet.rows <= 10 {
                viewport_row = 0;
            } else if viewport_row + 20 < sheet.rows {
                viewport_row += 10;
            } else {
                viewport_row = sheet.rows - 10;
            }
        } else if input_trimmed == "a" {
            viewport_col = viewport_col.saturating_sub(10);
        } else if input_trimmed == "d" {
            if sheet.cols <= 10 {
                viewport_col = 0;
            } else if viewport_col + 20 < sheet.cols {
                viewport_col += 10;
            } else {
                viewport_col = sheet.cols - 10;
            }
        } else if let Some(pos) = input_trimmed.find('=') {
            let (cell_str, expr) = input_trimmed.split_at(pos);
            let expr = &expr[1..]; // skip '='
                                   // Use Parser::label_to_coord for cell labels.
            if let Some((col, row)) = Parser::cell_name_to_coord(cell_str.trim()) {
                let message_code = sheet.set_cell((col, row), expr);
                which_message = message_code;
            } else {
                which_message = 1;
            }
        } else {
            which_message = 3;
        }
        if output_enabled {
            sheet.display(viewport_row, viewport_col, 10, 10);
        }
    }
}
