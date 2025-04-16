use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct Cell {
    pub formula: Option<String>,
    pub value: i32,
    // Instead of pointers, use indices.
    pub dependents: BTreeSet<usize>,
    pub precedents: BTreeSet<usize>,
}

impl Cell {
    pub fn new() -> Self {
        Self {
            formula: None,
            value: 0,
            dependents: BTreeSet::new(),
            precedents: BTreeSet::new(),
        }
    }
}

pub struct Spreadsheet {
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<Cell>,
}

impl Spreadsheet {
    /// Create a new spreadsheet with a flat vector of cells.
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            cells: vec![Cell::new(); rows * cols],
        }
    }

    /// Compute the flat index given (row, col).
    pub fn index(&self, row: usize, col: usize) -> usize {
        row * self.cols + col
    }

    /// Get an immutable reference to a cell.
    pub fn get_cell(&self, row: usize, col: usize) -> Option<&Cell> {
        if row < self.rows && col < self.cols {
            self.cells.get(self.index(row, col))
        } else {
            None
        }
    }

    /// Get a mutable reference to a cell.
    pub fn get_cell_mut(&mut self, row: usize, col: usize) -> Option<&mut Cell> {
        if row < self.rows && col < self.cols {
            // Compute the index locally to avoid borrowing self in a nested call.
            let idx = row * self.cols + col;
            self.cells.get_mut(idx)
        } else {
            None
        }
    }

    /// Display part of the spreadsheet.
    pub fn display(&self, start_row: usize, start_col: usize, max_rows: usize, max_cols: usize) {
        print!("    ");
        for c in start_col..(start_col + max_cols).min(self.cols) {
            print!("{:>12}", col_to_letter(c));
        }
        println!();

        for r in start_row..(start_row + max_rows).min(self.rows) {
            print!("{:3} ", r + 1);
            for c in start_col..(start_col + max_cols).min(self.cols) {
                let cell = self.get_cell(r, c).unwrap();
                if cell.value == crate::function::ERR_VALUE {
                    print!("{:>12}", "ERR");
                } else {
                    print!("{:>12}", cell.value);
                }
            }
            println!();
        }
    }
}

/// Convert a zero–based column index to its letter label (e.g., 0 → "A", 26 → "AA").
pub fn col_to_letter(mut col: usize) -> String {
    let mut letters = Vec::new();
    col += 1; // shift to one–based index
    while col > 0 {
        col -= 1;
        letters.push((b'A' + (col % 26) as u8) as char);
        col /= 26;
    }
    letters.iter().rev().collect()
}

/// Parse a cell name (e.g., "A1") into (row, col) with zero–based indices.
pub fn parse_cell_name(cell_name: &str) -> Option<(usize, usize)> {
    let mut chars = cell_name.chars().peekable();
    let mut col: usize = 0;
    let mut col_found = false;
    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_uppercase() {
            col_found = true;
            col = col * 26 + ((ch as u8 - b'A') as usize + 1);
            chars.next();
        } else {
            break;
        }
    }
    if !col_found {
        return None;
    }
    col -= 1; // zero–based

    let row_str: String = chars.collect();
    if row_str.is_empty() {
        return None;
    }
    if let Ok(row) = row_str.parse::<usize>() {
        if row == 0 {
            return None;
        }
        Some((row - 1, col))
    } else {
        None
    }
}
