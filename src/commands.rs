use std::io::{self, BufRead, Write};
use std::time::Instant;
use crate::spreadsheet::{Spreadsheet, parse_cell_name};
use crate::function::set_cell;

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
        "ok",
    ];
    let mut which_message: i32 = 0; // Use i32 for compatibility with set_cell.
    let mut last_instant = Instant::now();

    sheet.display(viewport_row, viewport_col, 10, 10);
    loop {
        let elapsed = last_instant.elapsed().as_secs_f64();
        // Cast which_message to usize for array indexing.
        print!("[{:.1}] ({}) > ", elapsed, status_messages[which_message as usize]);
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
        } else if input_trimmed == "disable_output" {
            output_enabled = false;
            continue;
        } else if input_trimmed == "enable_output" {
            output_enabled = true;
        } else if input_trimmed.starts_with("scroll_to") {
            let parts: Vec<&str> = input_trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Some((row, col)) = parse_cell_name(parts[1]) {
                    if row < sheet.rows && col < sheet.cols {
                        viewport_row = row;
                        viewport_col = col;
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
            if let Some((row, col)) = parse_cell_name(cell_str.trim()) {
                let mut msg: i32 = 0;
                set_cell(sheet, row, col, expr.trim(), &mut msg);
                which_message = msg;
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
