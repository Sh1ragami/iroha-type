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

    let header = Paragraph::new("[←/→]幅  [↑/↓]高  [+/-]ロスms  [F]固定打鍵  [[]/]]打鍵数  [C/X]CD秒  [M]サウンド  [O]音モード  [ESC]戻る  — 設定")
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(header, v[0]);

    let rows = vec![
        Row::new(vec![Cell::from("loss_ms_per_miss"), Cell::from(app.cfg.loss_ms_per_miss.to_string())]),
        Row::new(vec![Cell::from("theme"), Cell::from(app.cfg.theme.clone())]),
        Row::new(vec![Cell::from("stage_w"), Cell::from(app.cfg.stage_w.to_string())]),
        Row::new(vec![Cell::from("stage_h"), Cell::from(app.cfg.stage_h.to_string())]),
        Row::new(vec![Cell::from("fixed_chars"), Cell::from(app.cfg.fixed_chars.to_string())]),
        Row::new(vec![Cell::from("target_chars"), Cell::from(app.cfg.target_chars.to_string())]),
        Row::new(vec![Cell::from("countdown_sec"), Cell::from(app.cfg.countdown_sec.to_string())]),
        Row::new(vec![Cell::from("sound_enabled"), Cell::from(if app.cfg.sound_enabled { "true" } else { "false" })]),
        Row::new(vec![Cell::from("sound_mode"), Cell::from(match app.cfg.sound_mode { crate::util::config::SoundMode::Off=>"off", crate::util::config::SoundMode::Miss=>"miss", crate::util::config::SoundMode::All=>"all" })]),
        Row::new(vec![Cell::from("保存先"), Cell::from("data/")]),
    ];
    let table = Table::new(rows, [Constraint::Length(20), Constraint::Min(10)])
        .block(Block::default().borders(Borders::ALL).title("現在の設定"))
        .column_spacing(1);
    f.render_widget(table, v[1]);
}
