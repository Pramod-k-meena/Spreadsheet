use crate::function::{eval_binary, eval_range};
use crate::parser::Parser;
use std::collections::{HashMap, HashSet};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
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
    pub parents_normal: HashMap<(u16, u16), HashSet<(u16, u16)>>,
    pub child_normal: HashMap<(u16, u16), (String, HashSet<(u16, u16)>)>,
    pub child_range: HashMap<(u16, u16), (String, (u16, u16), (u16, u16))>,
    pub cells: Vec<Vec<Cell>>,
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

    /// Return `Some(v)` if cell is a value, or `None` if it's `Err` or out of bounds.
    fn get_val(&self, (c, r): (u16, u16)) -> Option<i32> {
        if r as usize <= self.rows && c as usize <= self.cols {
            match &self.cells[r as usize][c as usize] {
                Cell::Value(v) => Some(*v),
                Cell::Err => None,
            }
        } else {
            None
        }
    }

    /// Set a cell’s formula or literal.  Abort (no change) on any parse error,
    /// except when `/0` in a binary formula, which writes `Err`.
    pub fn set_cell(&mut self, coord: (u16, u16), expr: &str) -> u8 {
        if coord.1 as usize > self.rows || coord.0 as usize > self.cols {
            return 1; // Invalid cell
        }
        // Check for out-of-bounds coordinates
        //print parents and child dependencies before setting cell
        // println!("parents: {:?}", self.parents_normal);
        // println!("child: {:?}", self.child_normal);
        // println!("child_range: {:?}", self.child_range);
        // 1) clear old dependencies but save them first
        let old_cell_value = self.cells[coord.1 as usize][coord.0 as usize].clone();
        let mut removed_from_parents = Vec::new();
        let old_child_normal = self.child_normal.remove(&coord);
        let old_child_range = self.child_range.remove(&coord);

        // Track which entries we're removing from parents_normal
        for (parent_coord, deps) in self.parents_normal.iter_mut() {
            if deps.contains(&coord) {
                removed_from_parents.push((*parent_coord, coord));
                deps.remove(&coord);
            }
        }

        let expr = expr.trim();

        // 1a) Check for SLEEP function with a constant value: "SLEEP(5)"
        if expr.starts_with("SLEEP(") && expr.ends_with(")") {
            let arg_str = &expr[6..expr.len() - 1];
            // Check if the argument contains a colon, which indicates a range
            if arg_str.contains(':') {
                // Range is not allowed in SLEEP function
                // Restore old child dependencies
                if let Some(old_normal) = old_child_normal {
                    self.child_normal.insert(coord, old_normal);
                }
                if let Some(old_range) = old_child_range {
                    self.child_range.insert(coord, old_range);
                }
                // Restore parents
                for (parent_coord, child_coord) in removed_from_parents {
                    self.parents_normal
                        .entry(parent_coord)
                        .or_default()
                        .insert(child_coord);
                }
                return 3; // unrecognized cmd - range not allowed in SLEEP
            }
            // Try to parse as a literal integer
            if let Ok(sleep_time) = arg_str.parse::<i32>() {
                // Direct sleep with constant
                if sleep_time > 0 {
                    thread::sleep(Duration::from_secs(sleep_time as u64));
                }
                self.cells[coord.1 as usize][coord.0 as usize] = Cell::Value(sleep_time);
                self.recalc_dependents(coord);
                return 0;
            }
            // Try to parse as a cell reference
            else if let Some(ref_cell) = Parser::cell_name_to_coord(arg_str) {
                // Add dependency tracking
                self.parents_normal
                    .entry(ref_cell)
                    .or_default()
                    .insert(coord);
                let mut refs = HashSet::new();
                refs.insert(ref_cell);
                self.child_normal.insert(coord, (expr.to_string(), refs));

                if self.has_cycle() {
                    //reverse the parents normal and child normal changes done above
                    self.parents_normal
                        .entry(ref_cell)
                        .or_default()
                        .remove(&coord);
                    self.child_normal.remove(&coord);
                    //get the old values in these hashmaps
                    for (parent_coord, child_coord) in removed_from_parents {
                        self.parents_normal
                            .entry(parent_coord)
                            .or_default()
                            .insert(child_coord);
                    }
                    // Restore old child dependencies
                    if let Some(old_normal) = old_child_normal {
                        self.child_normal.insert(coord, old_normal);
                    }
                    if let Some(old_range) = old_child_range {
                        self.child_range.insert(coord, old_range);
                    }
                    return 4;
                }
                match self.get_val(ref_cell) {
                    Some(sleep_time) => {
                        // Sleep using the referenced cell's value
                        if sleep_time > 0 {
                            thread::sleep(Duration::from_secs(sleep_time as u64));
                        }
                        self.cells[coord.1 as usize][coord.0 as usize] = Cell::Value(sleep_time);
                    }
                    None => {
                        self.cells[coord.1 as usize][coord.0 as usize] = Cell::Err;
                    }
                }
                self.recalc_dependents(coord);
                return 0;
            }
        }

        // 2a) Binary: "A1+2", "3/0", etc.
        if let Some((op_char, lhs_s, rhs_s)) = Parser::split_binary(expr) {
            let op_code = match op_char {
                '+' => 1,
                '-' => 2,
                '*' => 3,
                '/' => 5,
                _ => return 3, //// unrecognized cmd  invalid operator
            };

            // Evaluate lhs
            let a = if let Some(c) = Parser::cell_name_to_coord(lhs_s) {
                match self.get_val(c) {
                    Some(val) => Cell::Value(val),
                    //for None return Err
                    None => Cell::Err,
                }
            } else {
                match lhs_s.parse::<i32>() {
                    Ok(val) => Cell::Value(val),
                    Err(_) => return 3,
                }
            };

            // Evaluate rhs
            let b = if let Some(c) = Parser::cell_name_to_coord(rhs_s) {
                match self.get_val(c) {
                    Some(val) => Cell::Value(val),
                    None => Cell::Err,
                }
            } else {
                match rhs_s.parse::<i32>() {
                    Ok(val) => Cell::Value(val),
                    Err(_) => return 3,
                }
            };

            let new_cell = if op_code == 5 && b == Cell::Value(0) {
                Cell::Err
            }
            //else if a or b is Err
            else if a == Cell::Err || b == Cell::Err {
                Cell::Err
            }
            //else if both are values
            else if let (Cell::Value(va), Cell::Value(vb)) = (a, b) {
                if let Some(v) = eval_binary(op_code, va, vb) {
                    Cell::Value(v)
                } else {
                    return 5; // division by zero
                }
            } else {
                return 3;
            };
            let mut updated_parents = Vec::new();
            // adding new dependencies
            let mut refs = HashSet::new();
            if let Some(c) = Parser::cell_name_to_coord(lhs_s) {
                updated_parents.push((c, coord));
                self.parents_normal.entry(c).or_default().insert(coord);
                refs.insert(c);
            }
            if let Some(c) = Parser::cell_name_to_coord(rhs_s) {
                updated_parents.push((c, coord));
                self.parents_normal.entry(c).or_default().insert(coord);
                refs.insert(c);
            }
            self.child_normal.insert(coord, (expr.to_string(), refs));
            // Check for cycles
            if self.has_cycle() {
                // Reverse the parents_normal and child_normal changes done above
                for (parent, child) in updated_parents {
                    self.parents_normal
                        .entry(parent)
                        .or_default()
                        .remove(&child);
                }
                self.child_normal.remove(&coord);
                // Restore old child dependencies
                if let Some(old_normal) = old_child_normal {
                    self.child_normal.insert(coord, old_normal);
                }
                if let Some(old_range) = old_child_range {
                    self.child_range.insert(coord, old_range);
                }
                // Restore parents
                for (parent_coord, child_coord) in removed_from_parents {
                    self.parents_normal
                        .entry(parent_coord)
                        .or_default()
                        .insert(child_coord);
                }
                // Keep the old cell value
                self.cells[coord.1 as usize][coord.0 as usize] = old_cell_value;
                return 4;
            }
            self.cells[coord.1 as usize][coord.0 as usize] = new_cell;
            self.recalc_dependents(coord);
            return 0;
        }

        // 2b) Range: "SUM(A1:B3)"
        if let Some((func, start, end)) = Parser::parse_range(expr) {
            // if start > end => return 3, and check in bounds
            if start.0 > end.0
                || start.1 > end.1
                || start.0 > self.cols as u16
                || start.1 > self.rows as u16
                || end.0 > self.cols as u16
                || end.1 > self.rows as u16
            {
                return 3; // unrecognized cmd
            }
            // Add the range dependency
            self.child_range
                .insert(coord, (expr.to_string(), start, end));

            // Check for cycles that might be created by this range reference
            if self.has_cycle() {
                // Cycle detected - remove the range dependency we just added
                self.child_range.remove(&coord);
                // Restore old child dependencies
                if let Some(old_normal) = old_child_normal {
                    self.child_normal.insert(coord, old_normal);
                }
                if let Some(old_range) = old_child_range {
                    self.child_range.insert(coord, old_range);
                }
                // Restore parents
                for (parent_coord, child_coord) in removed_from_parents {
                    self.parents_normal
                        .entry(parent_coord)
                        .or_default()
                        .insert(child_coord);
                }
                // Keep the old cell value
                self.cells[coord.1 as usize][coord.0 as usize] = old_cell_value;
                return 4;
            }
            // No cycle, proceed with evaluation
            if let Some(v) = eval_range(func, start, end, |c| self.get_val(c)) {
                self.cells[coord.1 as usize][coord.0 as usize] = Cell::Value(v);
            } else {
                self.cells[coord.1 as usize][coord.0 as usize] = Cell::Err;
            }
            self.recalc_dependents(coord);
            return 0;
        }

        // 2c) Single reference: "C5"
        if let Some(c) = Parser::cell_name_to_coord(expr) {
            // Add dependency tracking
            self.parents_normal.entry(c).or_default().insert(coord);
            let mut refs = HashSet::new();
            refs.insert(c);
            self.child_normal.insert(coord, (expr.to_string(), refs));

            // Check for cycles
            if self.has_cycle() {
                // Cycle detected - remove the dependency we just added
                self.parents_normal.entry(c).or_default().remove(&coord);
                self.child_normal.remove(&coord);
                // Restore old child dependencies
                if let Some(old_normal) = old_child_normal {
                    self.child_normal.insert(coord, old_normal);
                }
                if let Some(old_range) = old_child_range {
                    self.child_range.insert(coord, old_range);
                }
                // Restore parents
                for (parent_coord, child_coord) in removed_from_parents {
                    self.parents_normal
                        .entry(parent_coord)
                        .or_default()
                        .insert(child_coord);
                }
                // Keep the old cell value
                self.cells[coord.1 as usize][coord.0 as usize] = old_cell_value;
                return 4;
            }
            // No cycle, proceed with evaluation
            let v = self.get_val(c);
            match v {
                Some(val) => self.cells[coord.1 as usize][coord.0 as usize] = Cell::Value(val),
                None => self.cells[coord.1 as usize][coord.0 as usize] = Cell::Err,
            }
            self.recalc_dependents(coord);
            return 0;
        }

        // 2d) Literal: "42"
        if let Ok(v) = expr.parse::<i32>() {
            self.cells[coord.1 as usize][coord.0 as usize] = Cell::Value(v);
            self.recalc_dependents(coord);
            return 0;
        }

        // 2e) Anything else → abort with no change
        // RESTORE OLD CHILD DEPENDENCIES
        if let Some(old_normal) = old_child_normal {
            self.child_normal.insert(coord, old_normal);
        }
        if let Some(old_range) = old_child_range {
            self.child_range.insert(coord, old_range);
        }
        // Restore parents
        for (parent_coord, child_coord) in removed_from_parents {
            self.parents_normal
                .entry(parent_coord)
                .or_default()
                .insert(child_coord);
        }

        return 3; // unrecognized cmd
    }

    /// Recompute all dependents of `start`.  If division-by-zero occurs in a child,
    /// that child becomes `Err`; any other error in recomputation leaves it untouched.
    pub fn recalc_dependents(&mut self, start: (u16, u16)) {
        // Keep track of all cells that need to be recalculated
        let mut all_cells_to_update = Vec::new();
        let mut visited = HashSet::new();

        // Collect all cells affected by the change, including indirect dependencies
        let mut queue = vec![start];
        while let Some(cell) = queue.pop() {
            if !visited.insert(cell) {
                continue; // Skip if already visited
            }

            // Add to cells that need updating
            all_cells_to_update.push(cell);

            // Add normal dependents to queue
            if let Some(dependents) = self.parents_normal.get(&cell) {
                for &dependent in dependents {
                    if !visited.contains(&dependent) {
                        queue.push(dependent);
                    }
                }
            }

            // Check for range dependencies
            for (&range_cell, (_, range_start, range_end)) in &self.child_range {
                if is_within_range(cell, *range_start, *range_end) && !visited.contains(&range_cell)
                {
                    queue.push(range_cell);
                }
            }
        }

        // Now sort these cells topologically for correct calculation order
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        let mut topo_order = Vec::new();

        // Helper function to perform topological sort using DFS
        fn dfs(
            cell: (u16, u16),
            parents_normal: &HashMap<(u16, u16), HashSet<(u16, u16)>>,
            child_range: &HashMap<(u16, u16), (String, (u16, u16), (u16, u16))>,
            visited: &mut HashSet<(u16, u16)>,
            visiting: &mut HashSet<(u16, u16)>,
            topo_order: &mut Vec<(u16, u16)>,
        ) {
            if visited.contains(&cell) {
                return;
            }

            // Check for circular dependencies
            if !visiting.insert(cell) {
                // We've detected a cycle, just return without adding to topo_order
                return;
            }

            // Visit all dependents of this cell
            if let Some(dependents) = parents_normal.get(&cell) {
                for &dependent in dependents {
                    dfs(
                        dependent,
                        parents_normal,
                        child_range,
                        visited,
                        visiting,
                        topo_order,
                    );
                }
            }

            // Check range dependencies
            for (&range_cell, (_, range_start, range_end)) in child_range {
                if is_within_range(cell, *range_start, *range_end) && !visited.contains(&range_cell)
                {
                    dfs(
                        range_cell,
                        parents_normal,
                        child_range,
                        visited,
                        visiting,
                        topo_order,
                    );
                }
            }

            // Add current cell to result after all its dependents
            visiting.remove(&cell);
            visited.insert(cell);
            topo_order.push(cell);
        }

        // Build topological ordering from all affected cells
        for &cell in &all_cells_to_update {
            if !visited.contains(&cell) {
                dfs(
                    cell,
                    &self.parents_normal,
                    &self.child_range,
                    &mut visited,
                    &mut visiting,
                    &mut topo_order,
                );
            }
        }

        // Process cells in reverse topological order (dependencies before dependents)
        for cur in topo_order.iter().rev() {
            // Skip the start cell if it was already updated (e.g., by a set_cell call)
            if *cur == start {
                //this change fixed the issue of sleep (earlier it was *cur == start && all_cells_to_update.len() > 1)
                continue;
            }

            // compute new value for `cur`
            let new_cell = if let Some((formula, _)) = self.child_normal.get(cur).cloned() {
                // SLEEP function handling
                if formula.starts_with("SLEEP(") && formula.ends_with(")") {
                    let arg_str = &formula[6..formula.len() - 1];

                    // Try to parse as a literal integer
                    if let Ok(sleep_time) = arg_str.parse::<i32>() {
                        // Direct sleep with constant
                        if sleep_time > 0 {
                            thread::sleep(Duration::from_secs(sleep_time as u64));
                        }
                        Cell::Value(sleep_time)
                    }
                    // Try to parse as a cell reference
                    else if let Some(ref_cell) = Parser::cell_name_to_coord(arg_str) {
                        match self.get_val(ref_cell) {
                            Some(sleep_time) => {
                                // Sleep using the referenced cell's value
                                if sleep_time > 0 {
                                    thread::sleep(Duration::from_secs(sleep_time as u64));
                                    //this is the issue.
                                }
                                Cell::Value(sleep_time)
                            }
                            None => Cell::Err,
                        }
                    } else {
                        Cell::Err
                    }
                }
                // binary?
                else if let Some((op_char, lhs_s, rhs_s)) = Parser::split_binary(&formula) {
                    let op_code = match op_char {
                        '+' => 1,
                        '-' => 2,
                        '*' => 3,
                        '/' => 5,
                        _ => continue, // shouldn't happen
                    };

                    let a = if let Some(c) = Parser::cell_name_to_coord(lhs_s) {
                        self.get_val(c)
                    } else {
                        lhs_s.parse::<i32>().ok()
                    };
                    let b = if let Some(c) = Parser::cell_name_to_coord(rhs_s) {
                        self.get_val(c)
                    } else {
                        rhs_s.parse::<i32>().ok()
                    };

                    if op_code == 5 && b == Some(0) {
                        Cell::Err
                    } else if let (Some(a_val), Some(b_val)) = (a, b) {
                        if let Some(v) = eval_binary(op_code, a_val, b_val) {
                            Cell::Value(v)
                        } else {
                            Cell::Err
                        }
                    } else {
                        Cell::Err
                    }
                }
                // single‐cell ref?
                else if let Some(c) = Parser::cell_name_to_coord(&formula) {
                    match self.get_val(c) {
                        Some(val) => Cell::Value(val),
                        None => Cell::Err,
                    }
                }
                // literal?
                else if let Ok(v) = formula.parse::<i32>() {
                    Cell::Value(v)
                } else {
                    continue;
                }
            }
            // range?
            else if let Some((formula, range_start, range_end)) =
                self.child_range.get(cur).cloned()
            {
                let func = &formula[..formula.find('(').unwrap_or(0)];
                if let Some(v) = eval_range(func, range_start, range_end, |c| self.get_val(c)) {
                    Cell::Value(v)
                } else {
                    Cell::Err
                }
            } else {
                continue;
            };

            self.cells[cur.1 as usize][cur.0 as usize] = new_cell;
        }
    }

    /// Print a window of the sheet
    pub fn display(&self, start_row: usize, start_col: usize, max_rows: usize, max_cols: usize) {
        print!("    ");
        for c in (start_col + 1)..=(start_col + max_cols).min(self.cols) {
            print!("{:>8}", col_to_letter(c));
        }
        println!();

        for r in (start_row + 1)..=(start_row + max_rows).min(self.rows) {
            print!("{:>3} ", r);
            for c in (start_col + 1)..=(start_col + max_cols).min(self.cols) {
                match &self.cells[r][c] {
                    Cell::Value(v) => print!("{:>8}", v),
                    Cell::Err => print!("{:>8}", "ERR"),
                }
            }
            println!();
        }
    }

    // Add this new function to detect cycles in the dependency graph
    pub fn has_cycle(&self) -> bool {
        let mut visited = HashSet::new();
        let mut path = HashSet::new();

        // Create a vector to collect all cells we need to check
        let mut cells_to_check = Vec::new();

        // Add cells from parents_normal
        cells_to_check.extend(self.parents_normal.keys().copied());

        // Add cells from child_normal
        cells_to_check.extend(self.child_normal.keys().copied());

        // Add cells from range dependencies
        for (&c, (_, start, end)) in &self.child_range {
            cells_to_check.push(c);

            // Also add all cells within each range
            for col in start.0..=end.0 {
                for row in start.1..=end.1 {
                    cells_to_check.push((col, row));
                }
            }
        }

        // Remove duplicates
        cells_to_check.sort();
        cells_to_check.dedup();

        // Check each cell for cycles
        for cell in cells_to_check {
            if !visited.contains(&cell) {
                if self.is_cyclic(cell, &mut visited, &mut path) {
                    return true;
                }
            }
        }

        false
    }

    // Helper function for cycle detection using DFS
    fn is_cyclic(
        &self,
        cell: (u16, u16),
        visited: &mut HashSet<(u16, u16)>,
        path: &mut HashSet<(u16, u16)>,
    ) -> bool {
        visited.insert(cell);
        path.insert(cell);

        // Check normal dependencies
        if let Some(refs) = self.child_normal.get(&cell) {
            for &ref_cell in &refs.1 {
                if !visited.contains(&ref_cell) {
                    if self.is_cyclic(ref_cell, visited, path) {
                        return true;
                    }
                } else if path.contains(&ref_cell) {
                    // Found a cycle
                    return true;
                }
            }
        }

        // Check range dependencies
        if let Some((_, start, end)) = self.child_range.get(&cell) {
            for col in start.0..=end.0 {
                for row in start.1..=end.1 {
                    let ref_cell = (col, row);
                    if !visited.contains(&ref_cell) {
                        if self.is_cyclic(ref_cell, visited, path) {
                            return true;
                        }
                    } else if path.contains(&ref_cell) {
                        // Found a cycle
                        return true;
                    }
                }
            }
        }

        // Remove cell from current path as we backtrack
        path.remove(&cell);
        false
    }
}

// Helper function to check if a cell is within a range
fn is_within_range(cell: (u16, u16), start: (u16, u16), end: (u16, u16)) -> bool {
    let (col, row) = cell;
    let (start_col, start_row) = start;
    let (end_col, end_row) = end;

    // Make sure we have a properly ordered range
    let min_col = start_col.min(end_col);
    let max_col = start_col.max(end_col);
    let min_row = start_row.min(end_row);
    let max_row = start_row.max(end_row);

    // Check if the cell is within the range bounds
    col >= min_col && col <= max_col && row >= min_row && row <= max_row
}
