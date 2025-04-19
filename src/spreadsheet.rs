use crate::function::{eval_binary, eval_range, ERR_VALUE};
use crate::parser::Parser;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub enum Cell {
    Value(i32),
    Err,
}

impl Cell {
    pub fn new() -> Self {
        Cell::Value(0)
    }
}

/// Convert a 1-based column index into letters (1→"A", 27→"AA")
fn col_to_letter(mut n: usize) -> String {
    let mut s = String::new();
    while n > 0 {
        n -= 1;
        s.push((b'A' + (n % 26) as u8) as char);
        n /= 26;
    }
    s.chars().rev().collect()
}

pub struct Spreadsheet {
    pub rows: usize,
    pub cols: usize,
    /// parent→children map for normal formulas
    pub parents_normal: HashMap<(u16,u16), HashSet<(u16,u16)>>,
    /// (formula, refs) for normal formulas
    pub child_normal: HashMap<(u16,u16), (String, HashSet<(u16,u16)>)>,
    /// (formula, start, end) for range formulas
    pub child_range: HashMap<(u16,u16), (String, (u16,u16), (u16,u16))>,
    /// the grid, 1‑based indexing
    pub cells: Vec<Vec<Cell>>,
}

impl Spreadsheet {
    pub fn new(rows: usize, cols: usize) -> Self {
        // create (rows+1) × (cols+1) grid
        let mut cells = Vec::with_capacity(rows + 1);
        for _ in 0..=rows {
            cells.push(vec![Cell::new(); cols + 1]);
        }
        Spreadsheet {
            rows,
            cols,
            parents_normal: HashMap::new(),
            child_normal: HashMap::new(),
            child_range: HashMap::new(),
            cells,
        }
    }

    /// Fetch a cell’s i32 or ERR_VALUE if Err or OOB.
    fn get_val(&self, (c, r): (u16,u16)) -> i32 {
        if r as usize <= self.rows && c as usize <= self.cols {
            match &self.cells[r as usize][c as usize] {
                Cell::Value(v) => *v,
                Cell::Err      => ERR_VALUE,
            }
        } else {
            ERR_VALUE
        }
    }

    /// Set formula/value at `coord`, rebuild deps, and propagate.
    pub fn set_cell(&mut self, coord: (u16,u16), expr: &str) {
        // 1) clear old deps
        for deps in self.parents_normal.values_mut() {
            deps.remove(&coord);
        }
        self.child_normal.remove(&coord);
        self.child_range.remove(&coord);

        let expr = expr.trim();
        let result_cell = 
        // 2a) mixed or pure binary: "1+3", "B1+2", "B1+C1"
        if let Some((op_char, lhs_s, rhs_s)) = Parser::split_binary(expr) {
            let op_code = match op_char {
                '+' => 1, '-' => 2,
                '*' => 3, '/' => 5,
                _   => 0 
            };
            let mut refs = HashSet::new();
            // lhs
            let a = if let Some(c) = Parser::cell_name_to_coord(lhs_s) {
                refs.insert(c);
                self.parents_normal.entry(c).or_default().insert(coord);
                self.get_val(c)
            } else {
                lhs_s.parse().unwrap_or(ERR_VALUE)
            };
            // rhs
            let b = if let Some(c) = Parser::cell_name_to_coord(rhs_s) {
                refs.insert(c);
                self.parents_normal.entry(c).or_default().insert(coord);
                self.get_val(c)
            } else {
                rhs_s.parse().unwrap_or(ERR_VALUE)
            };
            // record formula & refs
            self.child_normal.insert(coord, (expr.to_string(), refs));
            let v = eval_binary(op_code, a, b);
            if v == ERR_VALUE { Cell::Err } else { Cell::Value(v) }
        }
        // 2b) range: "MAX(A1:B3)"
        else if let Some((func, s, e)) = Parser::parse_range(expr) {
            self.child_range.insert(coord, (expr.to_string(), s, e));
            let v = eval_range(func, s, e, |c| self.get_val(c));
            if v == ERR_VALUE { Cell::Err } else { Cell::Value(v) }
        }
        // 2c) single ref: "A1"
        else if let Some(c) = Parser::cell_name_to_coord(expr) {
            self.parents_normal.entry(c).or_default().insert(coord);
            let mut refs = HashSet::new();
            refs.insert(c);
            self.child_normal.insert(coord, (expr.to_string(), refs));
            let v = self.get_val(c);
            if v == ERR_VALUE { Cell::Err } else { Cell::Value(v) }
        }
        // 2d) pure literal: "42"
        else if let Ok(v) = expr.parse::<i32>() {
            Cell::Value(v)
        }
        // 2e) error
        else {
            Cell::Err
        };

        // 3) write & propagate
        self.cells[coord.1 as usize][coord.0 as usize] = result_cell;
        self.recalc_dependents(coord);
    }

    /// Recompute all dependents of `start`.
    pub fn recalc_dependents(&mut self, start: (u16,u16)) {
        let mut stack = vec![start];
        let mut seen = HashSet::new();
        while let Some(cur) = stack.pop() {
            if !seen.insert(cur) { continue; }
            // push children
            if let Some(children) = self.parents_normal.get(&cur) {
                for &ch in children { stack.push(ch) }
            }
            // recompute value
            let new_cell = 
            if let Some((formula, _)) = self.child_normal.get(&cur).cloned() {
                // reuse split_binary logic
                if let Some((op, lhs, rhs)) = Parser::split_binary(&formula) {
                    let a = Parser::cell_name_to_coord(lhs)
                        .map(|c| self.get_val(c))
                        .unwrap_or_else(|| lhs.parse().unwrap_or(ERR_VALUE));
                    let b = Parser::cell_name_to_coord(rhs)
                        .map(|c| self.get_val(c))
                        .unwrap_or_else(|| rhs.parse().unwrap_or(ERR_VALUE));
                    let v = eval_binary(op as i8, a, b);
                    if v == ERR_VALUE { Cell::Err } else { Cell::Value(v) }
                } else if let Some(c) = Parser::cell_name_to_coord(&formula) {
                    let v = self.get_val(c);
                    if v == ERR_VALUE { Cell::Err } else { Cell::Value(v) }
                } else if let Ok(v) = formula.parse::<i32>() {
                    Cell::Value(v)
                } else {
                    Cell::Err
                }
            }
            else if let Some((formula, s, e)) = self.child_range.get(&cur).cloned() {
                let func = &formula[..formula.find('(').unwrap_or(0)];
                let v = eval_range(func, s, e, |c| self.get_val(c));
                if v == ERR_VALUE { Cell::Err } else { Cell::Value(v) }
            }
            else {
                // no formula cell ⇒ skip
                continue;
            };

            self.cells[cur.1 as usize][cur.0 as usize] = new_cell;
        }
    }

    /// Print a 10×10 window starting at (start_row, start_col)
    pub fn display(&self, start_row: usize, start_col: usize, max_rows: usize, max_cols: usize) {
        // header
        print!("    ");
        for c in (start_col+1)..=(start_col+max_cols).min(self.cols) {
            print!("{:>8}", col_to_letter(c));
        }
        println!();

        for r in (start_row+1)..=(start_row+max_rows).min(self.rows) {
            print!("{:>3} ", r);
            for c in (start_col+1)..=(start_col+max_cols).min(self.cols) {
                match &self.cells[r][c] {
                    Cell::Value(v) => print!("{:>8}", v),
                    Cell::Err     => print!("{:>8}", "ERR"),
                }
            }
            println!();
        }
    }
}
