// use crate::parser::Parser;
// use crate::spreadsheet::Spreadsheet;
// use std::collections::HashSet;

// /// Register dependency relationships for a range function.
// /// For a target cell (the one holding the range function formula), add each cell in the range
// /// as having the target cell as a parent.
// pub fn register_range_dependencies(
//     sheet: &mut Spreadsheet,
//     target: (u16, u16),
//     range_str: &str,
// ) -> Result<(), i32> {
//     if let Some((_op, start, end)) = Parser::parse_range_formula(range_str) {
//         // Parser::label_to_coord returns (col, row) as 1-indexed.
//         let (start_col, start_row) = start;
//         let (end_col, end_row) = end;
//         // Iterate over the range: convert to zero-based indices for iteration, then back to 1-indexed keys.
//         for row in (start_row - 1)..=(end_row - 1) {
//             for col in (start_col - 1)..=(end_col - 1) {
//                 sheet.parents.entry((row+1,col+1)).or_default().insert(target);
//             }
//         }
//         Ok(())
//     } else {
//         Err(1)
//     }
// }

// /// Add a dependency: mark that the cell at `dependent` has the cell at `source` as a parent.
// pub fn add_dependent_and_precedent(sheet: &mut Spreadsheet, parent: (u16, u16), child: (u16, u16)) {
//     sheet.parents.entry(child).or_default().insert(parent);
// }

// /// Remove the dependency of `child` on `parent`.
// pub fn remove_dependent(sheet: &mut Spreadsheet, parent: (u16, u16), child: (u16, u16)) {
//     if let Some(parents_set) = sheet.parents.get_mut(&child) {
//         parents_set.remove(&parent);
//         if parents_set.is_empty() {
//             sheet.parents.remove(&child);
//         }
//     }
// }

// /// Remove all precedent (parent) dependencies from the given child cell.
// pub fn remove_all_precedents(sheet: &mut Spreadsheet, child: (u16, u16)) {
//     sheet.parents.remove(&child);
// }

// /// Returns true if there is a path from `from` → `to` in the current dependency graph.
// // fn has_cycle(sheet: &mut Spreadsheet, from: (u16, u16), to: (u16, u16), visited: &mut HashSet<(u16, u16)>) -> bool {
// //     // If we’ve reached the target, there’s a cycle.
// //     if from == to {
// //         return true;
// //     }
// //     // Avoid infinite loops
// //     if !visited.insert(from) {
// //         return false;
// //     }

// //     // Look at all formula‑cells that depend on `from`:
// //     if let Some(children) = self.parents.get(&from) {
// //         for &child in children {
// //             if self.has_path(child, to, visited) {
// //                 return true;
// //             }
// //         }
// //     }
// //     false
// // }


// /// Recalculate all dependent cells (i.e. those cells that have the given cell in their parent set)
// /// in a topologically sorted order. In this implementation, we compute dependents by traversing the
// /// dependency graph stored in `parents`. (You will need to implement a proper re-evaluation routine.)
// pub fn recalc_dependents(sheet: &mut Spreadsheet, start: (u16, u16)) {
//     let mut order = Vec::new();
//     let mut visited = HashSet::new();

//     // A DFS that collects cell keys in a post-order.
//     fn dfs(
//         sheet: &Spreadsheet,
//         node: (u16, u16),
//         visited: &mut HashSet<(u16, u16)>,
//         order: &mut Vec<(u16, u16)>,
//     ) {
//         for (&child, parents) in &sheet.parents {
//             if parents.contains(&node) && !visited.contains(&child) {
//                 visited.insert(child);
//                 dfs(sheet, child, visited, order);
//                 order.push(child);
//             }
//         }
//     }
//     dfs(sheet, start, &mut visited, &mut order);

//     // Here, you would re-evaluate each dependent cell's formula and update its value.
//     // For now, this is a placeholder demonstrating iteration over dependents.
//     for cell_key in order {
//         // e.g., recalc_cell(sheet, cell_key);
//     }
// }
