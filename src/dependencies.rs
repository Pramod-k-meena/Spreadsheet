use std::collections::BTreeSet;
use crate::spreadsheet::Spreadsheet;
use crate::function::set_only_value;

/// Add dependency: mark that `dependent_idx` depends on `source_idx`.
pub fn add_dependent_and_precedent(sheet: &mut Spreadsheet, source_idx: usize, dependent_idx: usize) {
    {
        let source = &mut sheet.cells[source_idx];
        source.dependents.insert(dependent_idx);
    }
    {
        let dependent = &mut sheet.cells[dependent_idx];
        dependent.precedents.insert(source_idx);
    }
}

/// Remove a dependent relationship.
pub fn remove_dependent(sheet: &mut Spreadsheet, main_idx: usize, dependent_idx: usize) {
    if let Some(cell) = sheet.cells.get_mut(main_idx) {
        cell.dependents.remove(&dependent_idx);
    }
}

/// Remove a precedent relationship.
pub fn remove_precedent(sheet: &mut Spreadsheet, main_idx: usize, precedent_idx: usize) {
    if let Some(cell) = sheet.cells.get_mut(main_idx) {
        cell.precedents.remove(&precedent_idx);
    }
}

/// Remove all precedents for the cell at index `cell_idx`.
pub fn remove_all_precedents(sheet: &mut Spreadsheet, cell_idx: usize) {
    let precedents: Vec<usize> = sheet.cells[cell_idx].precedents.iter().copied().collect();
    for &p in &precedents {
        remove_dependent(sheet, p, cell_idx);
    }
    sheet.cells[cell_idx].precedents.clear();
}

/// Check for a cycle starting at `start_idx` using DFS.
pub fn has_cycle(sheet: &Spreadsheet, start_idx: usize) -> bool {
    fn dfs(sheet: &Spreadsheet, cur: usize, start_idx: usize, visited: &mut BTreeSet<usize>) -> bool {
        if cur == start_idx && !visited.is_empty() {
            return true;
        }
        if !visited.insert(cur) {
            return false;
        }
        let cell = &sheet.cells[cur];
        for &dep in &cell.dependents {
            if dfs(sheet, dep, start_idx, visited) {
                return true;
            }
        }
        false
    }
    let mut visited = BTreeSet::new();
    dfs(sheet, start_idx, start_idx, &mut visited)
}

/// Recalculate all dependent cells (in topologically–sorted order) starting from `start_idx`.
pub fn recalc_dependents(sheet: &mut Spreadsheet, start_idx: usize) {
    fn dfs(sheet: &Spreadsheet, cur: usize, visited: &mut BTreeSet<usize>, order: &mut Vec<usize>) {
        if visited.contains(&cur) {
            return;
        }
        visited.insert(cur);
        for &dep in &sheet.cells[cur].dependents {
            dfs(sheet, dep, visited, order);
        }
        order.push(cur);
    }
    let mut visited = BTreeSet::new();
    let mut order = Vec::new();
    dfs(sheet, start_idx, &mut visited, &mut order);
    order.reverse();
    for idx in order {
        set_only_value(sheet, idx);
    }
}
