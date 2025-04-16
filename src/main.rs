mod spreadsheet;
mod dependencies;
mod function;
mod commands;

use std::env;
use spreadsheet::Spreadsheet;
use commands::handle_commands;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <rows> <cols>", args[0]);
        return;
    }
    let rows: usize = args[1].parse().unwrap_or(0);
    let cols: usize = args[2].parse().unwrap_or(0);
    if rows < 1 || rows > 999 || cols < 1 || cols > 18278 {
        eprintln!("Error: Invalid rows or columns. Rows must be between 1 and 999, columns between 1 and 18278.");
        return;
    }
    let mut sheet = Spreadsheet::new(rows, cols);
    handle_commands(&mut sheet);
}
