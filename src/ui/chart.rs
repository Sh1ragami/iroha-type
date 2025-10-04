use ratatui::{prelude::*, widgets::*};
use crate::store::json::ScoreRecord;

pub fn draw_speed_chart(f: &mut Frame, area: Rect, rec: &ScoreRecord) {
    // Build datasets: speed points and moving average (2.0s window)
    let pts: Vec<(f64,f64)> = rec.speed_series.clone().unwrap_or_default();
    let ma = moving_average(&pts, 2.0);
    let data1: Vec<(f64,f64)> = pts.clone();
    let data2: Vec<(f64,f64)> = ma;

    let max_x = data1.last().map(|p| p.0).unwrap_or(1.0).max(1.0);
    let max_y = data1.iter().chain(data2.iter()).map(|p| p.1).fold(1.0, f64::max);

    let ds1 = Dataset::default().name("Speed").marker(symbols::Marker::Dot).style(Style::default().fg(Color::Cyan)).data(&data1);
    let ds2 = Dataset::default().name("MA(2s)").marker(symbols::Marker::Braille).style(Style::default().fg(Color::Yellow)).data(&data2);

    let chart = Chart::new(vec![ds1, ds2])
        .block(Block::default().title("速度推移").borders(Borders::ALL))
        .x_axis(Axis::default().title("sec").bounds([0.0, max_x]))
        .y_axis(Axis::default().title("keystrokes/s").bounds([0.0, max_y * 1.1]));
    f.render_widget(chart, area);
}

fn moving_average(pts: &[(f64,f64)], window_sec: f64) -> Vec<(f64,f64)> {
    let mut out = Vec::with_capacity(pts.len());
    for (i, (tx, _)) in pts.iter().enumerate() {
        let from = tx - window_sec;
        let mut sum = 0.0; let mut cnt = 0.0;
        for j in (0..=i).rev() {
            let (x, y) = pts[j];
            if x < from { break; }
            sum += y; cnt += 1.0;
        }
        out.push((*tx, if cnt>0.0 { sum/cnt } else { 0.0 }));
    }
    out
}

