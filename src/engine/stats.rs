use super::game::Split;

pub fn compute_wpm_stats(splits: &[Split]) -> (f64, f64) {
    let mut top = 0.0_f64;
    let mut worst = f64::MAX;
    for s in splits {
        if s.sec <= 0.0 || s.keystrokes == 0 { continue; }
        let wpm = s.keystrokes as f64 / s.sec * 60.0;
        if wpm > top { top = wpm; }
        if wpm < worst { worst = wpm; }
    }
    if worst==f64::MAX { worst = 0.0; }
    (top, worst)
}

pub fn moving_speed_series(points: &[(f64,f64)], _window: f64) -> Vec<(f64,f64)> {
    // Unused: we compute moving average in ui::chart directly
    points.to_vec()
}

