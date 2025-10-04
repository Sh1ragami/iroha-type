use ratatui::{prelude::*, widgets::*};
use crate::app::App;

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.size();
    let stage = super::centered(area, app.cfg.stage_w.into(), app.cfg.stage_h.into());
    f.render_widget(Clear, area);
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(4), Constraint::Min(6)])
        .split(stage);

    let header = Paragraph::new(Line::from(vec![
        Span::styled("[Enter]ランキング", Style::default().fg(Color::Yellow)),
        Span::raw(" / "),
        Span::styled("[ESC]戻る", Style::default().fg(Color::Magenta)),
        Span::raw("  — 結果"),
    ]))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(header, v[0]);

    if let Some(rec) = &app.last_result {
        // 概要を2列表形式でコンパクト表示
        let rows = vec![
            Row::new(vec![Cell::from("Time(s)"), Cell::from(format!("{:.3}", rec.time_sec))]),
            Row::new(vec![Cell::from("Miss"), Cell::from(format!("{}", rec.miss))]),
            Row::new(vec![Cell::from("Loss(s)"), Cell::from(format!("{:.3}", rec.timeloss_sec))]),
            Row::new(vec![Cell::from("Top/ Worst WPM"), Cell::from(format!("{:.1} / {:.1}", rec.wpm_top, rec.wpm_worst))]),
            Row::new(vec![Cell::from("Rank"), Cell::from(rec.rank.clone())]),
        ];
        let table = Table::new(rows, [Constraint::Length(14), Constraint::Min(10)])
            .block(Block::default().borders(Borders::ALL).title("概要"))
            .column_spacing(1);
        f.render_widget(table, v[1]);

        // チャートは残り全域
        super::chart::draw_speed_chart(f, v[2], rec);
    }
}
