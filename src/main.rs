mod commands;
mod function;
mod parser;
mod spreadsheet;

use commands::handle_commands;
use spreadsheet::Spreadsheet;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <rows> <cols>", args[0]);
        std::process::exit(1);
    }
    let rows: usize = args[1].parse().unwrap_or(0);
    let cols: usize = args[2].parse().unwrap_or(0);
    if rows < 1 || rows > 999 || cols < 1 || cols > 18278 {
        eprintln!(
            "Error: Invalid rows or cols; got {}x{}. Valid: 1≤rows≤999, 1≤cols≤18278.",
            rows, cols
        );
        std::process::exit(1);
    }
    let mut sheet = Spreadsheet::new(rows, cols);
    handle_commands(&mut sheet);
}
