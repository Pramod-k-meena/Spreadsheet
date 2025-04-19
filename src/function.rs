pub const ERR_VALUE: i32 = -999_999;

/// Evaluate a pure or mixed binary expression, given op_code and two i32 operands.
pub fn eval_binary(op: i8, a: i32, b: i32) -> i32 {
    match op {
        1 => a + b,
        2 => a - b,
        3 => a * b,
        5 => if b == 0 { ERR_VALUE } else { a / b },
        _ => ERR_VALUE,
    }
}

/// Evaluate a range function over start..=end using get_val callback.
pub fn eval_range(
    func: &str,
    start: (u16, u16),
    end: (u16, u16),
    get_val: impl Fn((u16, u16)) -> i32,
) -> i32 {
    let mut vals = Vec::new();
    for c in start.0..=end.0 {
        for r in start.1..=end.1 {
            let v = get_val((c, r));
            if v == ERR_VALUE {
                return ERR_VALUE;
            }
            vals.push(v as f64);
        }
    }
    if vals.is_empty() {
        return 0;
    }
    match func {
        "MIN" => vals.into_iter().fold(f64::INFINITY, f64::min) as i32,
        "MAX" => vals.into_iter().fold(f64::NEG_INFINITY, f64::max) as i32,
        "AVG" => (vals.iter().sum::<f64>() / vals.len() as f64).round() as i32,
        "SUM" => vals.iter().sum::<f64>() as i32,
        "STDEV" => {
            let mean = vals.iter().sum::<f64>() / vals.len() as f64;
            let var = vals.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / vals.len() as f64;
            var.sqrt().round() as i32
        }
        "SLEEP" => 0,
        _ => ERR_VALUE,
    }
}
