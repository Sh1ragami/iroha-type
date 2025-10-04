use ratatui::{prelude::*, widgets::*};
use crate::app::App;

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.size();
    let stage = super::centered(area, app.cfg.stage_w.into(), app.cfg.stage_h.into());
    f.render_widget(Clear, area);
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(stage);

    let header = Paragraph::new(Line::from(vec![
        Span::styled("[ESC]戻る", Style::default().fg(Color::Magenta)),
        Span::raw("  — ランキング"),
    ]))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(header, v[0]);

    // 本文は左右2カラム: 左=TOP15, 右=Lap10
    let h = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(v[1]);

    // TOP15 テーブル
    let rows_top: Vec<Row> = app
        .scorebook
        .top
        .iter()
        .take(15)
        .enumerate()
        .map(|(i, r)| Row::new(vec![
            Cell::from(format!("{:>2}", i + 1)),
            Cell::from(format!("{:>6.1}", r.wpm_top)),
            Cell::from(format!("{:>3}", r.miss)),
            Cell::from(format!("{:>7.3}", r.time_sec)),
            Cell::from(truncate(&r.datetime, (h[0].width as usize).saturating_sub(28))),
        ]))
        .collect();
    let table_top = Table::new(rows_top, [
            Constraint::Length(3),
            Constraint::Length(8),
            Constraint::Length(5),
            Constraint::Length(9),
            Constraint::Min(10),
        ])
        .block(Block::default().borders(Borders::ALL).title("TOP 15"))
        .header(Row::new(vec!["#", "WPM", "Miss", "Time(s)", "Date"]).style(Style::default().fg(Color::Yellow)))
        .column_spacing(1);
    f.render_widget(table_top, h[0]);

    // Lap10 テーブル
    let rows_lap: Vec<Row> = app
        .scorebook
        .lap
        .iter()
        .take(10)
        .enumerate()
        .map(|(i, r)| Row::new(vec![
            Cell::from(format!("{:>2}", i + 1)),
            Cell::from(format!("{:>7.3}", r.time_sec)),
            Cell::from(truncate(r.word_display.as_deref().unwrap_or(""), (h[1].width as usize).saturating_sub(14))),
        ]))
        .collect();
    let table_lap = Table::new(rows_lap, [Constraint::Length(3), Constraint::Length(9), Constraint::Min(6)])
        .block(Block::default().borders(Borders::ALL).title("Lap 10"))
        .header(Row::new(vec!["#", "秒", "語"]).style(Style::default().fg(Color::Yellow)))
        .column_spacing(1);
    f.render_widget(table_lap, h[1]);
}

fn truncate(s: &str, max_w: usize) -> String {
    let w = unicode_width::UnicodeWidthStr::width(s);
    if w <= max_w { return s.to_string(); }
    let mut out = String::new();
    let mut cur = 0;
    for ch in s.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
        if cur + cw > max_w.saturating_sub(1) { break; }
        cur += cw; out.push(ch);
    }
    out.push('…');
    out
}
