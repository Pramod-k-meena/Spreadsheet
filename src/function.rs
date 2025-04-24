use std::thread;
use std::time::Duration;

/// Evaluate a simple binary operation.  
/// Returns `Some(result)` or `None` if the operator is invalid or if division by zero is attempted.
pub fn eval_binary(op: i8, a: i32, b: i32) -> Option<i32> {
    match op {
        1 => Some(a + b),
        2 => Some(a - b),
        3 => Some(a * b),
        5 => {
            if b == 0 {
                None
            } else {
                Some(a / b)
            }
        }
        _ => None,
    }
}

/// Calculate the minimum value in the specified range.
/// Returns `None` if any cell is in an error state.
pub fn min_range<F>(
    start: (u16, u16),
    end: (u16, u16),
    get_val: F,
) -> Option<i32>
where
    F: Fn((u16, u16)) -> Option<i32>,
{
    let mut min_val = i32::MAX;
    for c in start.0..=end.0 {
        for r in start.1..=end.1 {
            let v = get_val((c, r))?;
            if v < min_val {
                min_val = v;
            }
        }
    }
    Some(min_val)
}

/// Calculate the maximum value in the specified range.
/// Returns `None` if any cell is in an error state.
pub fn max_range<F>(
    start: (u16, u16),
    end: (u16, u16),
    get_val: F,
) -> Option<i32>
where
    F: Fn((u16, u16)) -> Option<i32>,
{
    let mut max_val = i32::MIN;
    for c in start.0..=end.0 {
        for r in start.1..=end.1 {
            let v = get_val((c, r))?;
            if v > max_val {
                max_val = v;
            }
        }
    }
    Some(max_val)
}

/// Calculate the average (rounded down) of values in the specified range.
/// Returns `None` if any cell is in an error state, or `Some(0)` if the range is empty.
pub fn avg_range<F>(
    start: (u16, u16),
    end: (u16, u16),
    get_val: F,
) -> Option<i32>
where
    F: Fn((u16, u16)) -> Option<i32>,
{
    let mut sum: i64 = 0;
    let mut count: i64 = 0;
    for c in start.0..=end.0 {
        for r in start.1..=end.1 {
            let v = get_val((c, r))?;
            sum += v as i64;
            count += 1;
        }
    }
    if count == 0 {
        Some(0)
    } else {
        Some((sum / count) as i32)
    }
}

/// Calculate the sum of values in the specified range.
/// Returns `None` if any cell is in an error state.
pub fn sum_range<F>(
    start: (u16, u16),
    end: (u16, u16),
    get_val: F,
) -> Option<i32>
where
    F: Fn((u16, u16)) -> Option<i32>,
{
    let mut sum: i32 = 0;
    for c in start.0..=end.0 {
        for r in start.1..=end.1 {
            let v = get_val((c, r))?;
            sum += v;
        }
    }
    Some(sum)
}

/// Calculate the standard deviation (rounded) of values in the specified range.
/// Returns `None` if any cell is in an error state, or `Some(0)` if fewer than 2 cells.
pub fn stdev_range<F>(
    start: (u16, u16),
    end: (u16, u16),
    get_val: F,
) -> Option<i32>
where
    F: Fn((u16, u16)) -> Option<i32>,
{
    // First pass: sum and count
    let mut sum: f64 = 0.0;
    let mut count: usize = 0;
    for c in start.0..=end.0 {
        for r in start.1..=end.1 {
            let v = get_val((c, r))? as f64;
            sum += v;
            count += 1;
        }
    }
    if count <= 1 {
        return Some(0);
    }
    let mean = sum / count as f64;

    // Second pass: accumulate squared deviations
    let mut var_sum: f64 = 0.0;
    for c in start.0..=end.0 {
        for r in start.1..=end.1 {
            let v = get_val((c, r))? as f64;
            let diff = v - mean;
            var_sum += diff * diff;
        }
    }
    let variance = var_sum / count as f64;
    Some(variance.sqrt().round() as i32)
}

/// Evaluate a range function (MIN/MAX/AVG/SUM/STDEV/SLEEP).
/// The callback returns `Some(value)` for each cell, or `None` to signal an error.
/// Returns `Some(aggregate)` or `None`.
pub fn eval_range<F>(
    func: &str,
    start: (u16, u16),
    end: (u16, u16),
    get_val: F,
) -> Option<i32>
where
    F: Fn((u16, u16)) -> Option<i32>,
{
    // Special handling for SLEEP function
    if func.eq_ignore_ascii_case("SLEEP") {
        let sec = get_val(start)?;
        if sec > 0 {
            thread::sleep(Duration::from_secs(sec as u64));
        }
        return Some(sec);
    }

    // Dispatch to the appropriate helper
    match func.to_uppercase().as_str() {
        "MIN" => min_range(start, end, &get_val),
        "MAX" => max_range(start, end, &get_val),
        "AVG" => avg_range(start, end, &get_val),
        "SUM" => sum_range(start, end, &get_val),
        "STDEV" => stdev_range(start, end, &get_val),
        _ => None,
    }
}
