use ratatui::{prelude::*, widgets::*};
use crate::app::App;

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.size();
    let stage = super::centered(area, app.cfg.stage_w.into(), app.cfg.stage_h.into());
    f.render_widget(Clear, area);
    // 上: TOP 15 RANKING + 右パネル, 下: TOP Lap 10
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Percentage(60),
            Constraint::Percentage(30),
            Constraint::Length(1),
        ])
        .split(stage);

    f.render_widget(Paragraph::new("[ESC] 戻る  —  RANKING"), v[0]);

    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(78), Constraint::Percentage(22)])
        .split(v[1]);

    // TOP 15 (Time-centric)
    let rows_top: Vec<Row> = app.scorebook.top.iter().take(15).enumerate().map(|(i,r)|{
        Row::new(vec![
            Cell::from(format!("{:>2}", i+1)),
            Cell::from(format!("{:>7.3}", r.time_sec)),
            Cell::from(r.rank.clone()),
            Cell::from(format!("{:>3}", r.miss)),
            Cell::from(truncate(&r.datetime, (top_cols[0].width as usize).saturating_sub(28))),
        ]).style(Style::default().fg(Color::White))
    }).collect();
    let table_top = Table::new(rows_top, [
        Constraint::Length(4), // Rk
        Constraint::Length(10), // Time
        Constraint::Length(4), // Lv
        Constraint::Length(5), // Ms
        Constraint::Min(12),   // Date
    ])
    .block(Block::default().borders(Borders::ALL).title(Span::styled(" TOP 15  RANKING ", Style::default().fg(Color::White).bg(Color::Blue).add_modifier(Modifier::BOLD))))
    .header(Row::new(vec!["Rk","Time","Lv","Ms","Date"]).style(Style::default().fg(Color::Yellow)))
    .column_spacing(1);
    f.render_widget(table_top, top_cols[0]);

    // Right panel: summary for last result
    let mut panel_lines: Vec<Line> = Vec::new();
    if let Some(rec) = &app.last_result {
        panel_lines.push(Line::from(Span::styled("Summary", Style::default().fg(Color::Cyan))));
        panel_lines.push(Line::from(format!("Time  {:>7.3}s", rec.time_sec)));
        panel_lines.push(Line::from(format!("Lv    {}", rec.rank)));
        panel_lines.push(Line::from(format!("Miss  {}", rec.miss)));
        panel_lines.push(Line::from(""));
        panel_lines.push(Line::from(Span::styled("Possible", Style::default().fg(Color::Gray))));
        panel_lines.push(Line::from(format!("{:>7.3}s", (rec.time_sec - rec.timeloss_sec).max(0.0))));
    } else {
        panel_lines.push(Line::from("No recent record"));
    }
    let panel = Paragraph::new(panel_lines)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(panel, top_cols[1]);

    // BOTTOM: TOP Lap 10 across columns 1..10
    let mut lap_cells: Vec<Span> = Vec::new();
    for (i, r) in app.scorebook.lap.iter().take(10).enumerate() {
        if i>0 { lap_cells.push(Span::raw("  ")); }
        lap_cells.push(Span::styled(format!("{:>1}", i+1), Style::default().fg(Color::Gray)));
        lap_cells.push(Span::raw(" "));
        lap_cells.push(Span::styled(format!("{:>5.3}", r.time_sec), Style::default().fg(Color::Red)));
    }
    let lap_block = Paragraph::new(Line::from(lap_cells))
        .block(Block::default().borders(Borders::ALL).title(Span::styled(" TOP Lap 10 ", Style::default().fg(Color::White).bg(Color::Blue).add_modifier(Modifier::BOLD))));
    f.render_widget(lap_block, v[2]);

    // Footer line
    f.render_widget(Paragraph::new("OK: ESC"), v[3]);
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
