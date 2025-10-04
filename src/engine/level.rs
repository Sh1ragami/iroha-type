pub fn estimate_rank(cps: f64) -> &'static str {
    // Rough thresholds (keys per second) mapped to TypeWell-like ranks
    if cps >= 13.0 { "XS" }
    else if cps >= 11.0 { "XD" }
    else if cps >= 9.5 { "XC" }
    else if cps >= 8.0 { "SA" }
    else if cps >= 7.0 { "SB" }
    else if cps >= 6.0 { "A" }
    else if cps >= 5.0 { "B" }
    else if cps >= 4.0 { "C" }
    else if cps >= 3.0 { "D" }
    else if cps >= 2.0 { "E" }
    else { "F" }
}

