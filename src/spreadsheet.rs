use crate::function::{eval_binary, eval_range, ERR_VALUE};
use crate::parser::Parser;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct Cell {
    pub value: i32,
}

impl Cell {
    pub fn new() -> Self {
        Cell { value: 0 }
    }
}

pub struct Spreadsheet {
    pub rows: usize,
    pub cols: usize,
    /// For each parent cell → set of children (the cells that depend on it)
    pub parents_normal: HashMap<(u16, u16), HashSet<(u16, u16)>>,
    /// For each formula cell (normal) → (formula_str, {referenced cells})
    pub child_normal: HashMap<(u16, u16), (String, HashSet<(u16, u16)>)>,
    /// For each formula cell (range) → (formula_str, start, end)
    pub child_range: HashMap<(u16, u16), (String, (u16, u16), (u16, u16))>,
    pub cells: Vec<Vec<Cell>>, // 1-based: indexed as [row][col]
}

impl Spreadsheet {
    pub fn new(rows: usize, cols: usize) -> Self {
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

    /// Set a cell’s formula or value, update dependency maps, and recalc dependents.
    pub fn set_cell(&mut self, coord: (u16, u16), expr: &str) {
        // 1) clear old deps
        for deps in self.parents_normal.values_mut() {
            deps.remove(&coord);
        }
        self.child_normal.remove(&coord);
        self.child_range.remove(&coord);

        let expr = expr.trim();

        // 2) evaluate & rebuild maps
        let val =    // — first: pure integer literal
        if let Ok(v) = expr.parse::<i32>() {
            v
        }
        // — next: single‑cell reference A1 = B1
        else if let Some(c) = Parser::cell_name_to_coord(expr) {
            self.parents_normal.entry(c).or_default().insert(coord);
            let mut set = HashSet::new();
            set.insert(c);
            self.child_normal.insert(coord, (expr.to_string(), set));
            self.get_val(c)
        }
        // — next: range formulas MAX(A1:B2)
        else if let Some((func, start, end)) = Parser::parse_range(expr) {
            // for c in start.0..=end.0 {
            //     for r in start.1..=end.1 {
            //         self.parents_normal.entry((c, r)).or_default().insert(coord);
            //     }
            // }
            self.child_range
                .insert(coord, (expr.to_string(), start, end));
            eval_range(func, start, end, |c| self.get_val(c))
        }
        // — finally: binary/mixed A1=B1+2 or A1=2+3
        else if let Some((op, lhs_s, rhs_s)) = Parser::split_binary(expr) {
            let mut referenced = HashSet::new();
            // lhs
            let a = if let Some(c) = Parser::cell_name_to_coord(lhs_s) {
                referenced.insert(c);
                self.parents_normal.entry(c).or_default().insert(coord);
                self.get_val(c)
            } else {
                lhs_s.parse().unwrap_or(ERR_VALUE)
            };
            // rhs
            let b = if let Some(c) = Parser::cell_name_to_coord(rhs_s) {
                referenced.insert(c);
                self.parents_normal.entry(c).or_default().insert(coord);
                self.get_val(c)
            } else {
                rhs_s.parse().unwrap_or(ERR_VALUE)
            };
            if !referenced.is_empty() {
                self.child_normal
                    .insert(coord, (expr.to_string(), referenced));
            }
            eval_binary(op, a, b)
        } else {
            // anything else is an error
            ERR_VALUE
        };

        // 3) write & propagate
        self.cells[coord.1 as usize][coord.0 as usize].value = val;
        self.recalc_dependents(coord);
    }


    /// Recalculate all cells that (directly or indirectly) depend on `start`.
    pub fn recalc_dependents(&mut self, start: (u16, u16)) {
        let mut stack = vec![start];
        let mut seen = HashSet::new();

        while let Some(cur) = stack.pop() {
            if !seen.insert(cur) {
                continue;
            }

            // push children
            if let Some(deps) = self.parents_normal.get(&cur) {
                for &ch in deps {
                    stack.push(ch);
                }
            }

            // recalc cell `cur` if it has a formula
            if let Some((formula, _refs)) = self.child_normal.get(&cur).cloned() {
                // re‑evaluate via same logic as set_cell
                let new_val = {
                    if let Some((op, lhs_s, rhs_s)) = Parser::split_binary(&formula) {
                        let a = Parser::cell_name_to_coord(lhs_s)
                            .map(|c| self.get_val(c))
                            .unwrap_or_else(|| lhs_s.parse().unwrap_or(ERR_VALUE));
                        let b = Parser::cell_name_to_coord(rhs_s)
                            .map(|c| self.get_val(c))
                            .unwrap_or_else(|| rhs_s.parse().unwrap_or(ERR_VALUE));
                        eval_binary(op, a, b)
                    } else if let Some(c) = Parser::cell_name_to_coord(&formula) {
                        self.get_val(c)
                    } else if let Ok(v) = formula.parse::<i32>() {
                        v
                    } else {
                        ERR_VALUE
                    }
                };
                self.cells[cur.1 as usize][cur.0 as usize].value = new_val;
            } else if let Some((formula, start, end)) = self.child_range.get(&cur).cloned() {
                let new_val = eval_range(&formula[..formula.find('(').unwrap_or(0)], start, end, |c| {
                    self.get_val(c)
                });
                self.cells[cur.1 as usize][cur.0 as usize].value = new_val;
            }
        }
    }

    /// Fetch a cell’s current value (1‑based coord) or ERR_VALUE if OOB.
    fn get_val(&self, (c, r): (u16, u16)) -> i32 {
        if r as usize <= self.rows && c as usize <= self.cols {
            self.cells[r as usize][c as usize].value
        } else {
            ERR_VALUE
        }
    }

    /// Display a viewport of the sheet.
    pub fn display(&self, start_row: usize, start_col: usize, max_rows: usize, max_cols: usize) {
        // header
        print!("    ");
        for c in (start_col + 1)..=(start_col + max_cols).min(self.cols) {
            print!("{:>8}", col_to_letter(c));
        }
        println!();

        for r in (start_row + 1)..=(start_row + max_rows).min(self.rows) {
            print!("{:>3} ", r);
            for c in (start_col + 1)..=(start_col + max_cols).min(self.cols) {
                let v = self.cells[r][c].value;
                if v == ERR_VALUE {
                    print!("{:>8}", "ERR");
                } else {
                    print!("{:>8}", v);
                }
            }
            println!();
        }
    }
}

/// Column index (1→"A", 27→"AA")
fn col_to_letter(mut n: usize) -> String {
    let mut s = String::new();
    while n > 0 {
        n -= 1;
        s.push((b'A' + (n % 26) as u8) as char);
        n /= 26;
    }
    s.chars().rev().collect()
}
