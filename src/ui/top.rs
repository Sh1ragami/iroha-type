use ratatui::{prelude::*, widgets::*};
use crate::app::App;

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.size();
    // Centered stage (leave margins blank around)
    let stage = super::centered(area, app.cfg.stage_w.into(), app.cfg.stage_h.saturating_sub(6).max(12).into());
    f.render_widget(Clear, area); // clear whole screen to ensure margins are blank
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // key guide
            Constraint::Min(3),     // title area
            Constraint::Length(1),  // footer
        ])
        .split(stage);

    let guide = Line::from(vec![
        Span::styled("[G] Start ", Style::default().fg(Color::Green)),
        Span::styled("[R] Ranking ", Style::default().fg(Color::Yellow)),
        Span::styled("[S] Settings ", Style::default().fg(Color::Cyan)),
        Span::styled("[Q] Quit", Style::default().fg(Color::Red)),
    ]);
    f.render_widget(Paragraph::new(guide), v[0]);

    let center = v[1];
    let title_line = Line::from(gradient_spans_static(&app.cfg.app_name));
    let tag = Line::from("Japanese Typing TUI").style(Style::default().fg(Color::Gray));
    let body = vec![title_line, tag];
    let title = Paragraph::new(body)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, center);

    let foot = if let Some(r) = &app.last_result {
        Paragraph::new(Line::from(format!(
            "Last  {:.3}s  Miss {}  Rank {}  Top {:.1} WPM",
            r.time_sec, r.miss, r.rank, r.wpm_top
        ))).alignment(Alignment::Center)
    } else {
        Paragraph::new(Line::from("Press G to begin.")).alignment(Alignment::Center)
    };
    f.render_widget(foot, v[2]);
}

fn palette() -> [Color; 6] {
    [Color::LightCyan, Color::Cyan, Color::LightBlue, Color::White, Color::Gray, Color::LightMagenta]
}

fn gradient_spans_static(text: &str) -> Vec<Span<'static>> {
    // Soft, non-animated gradient (fixed offset)
    let pal = palette();
    let off = 2usize; // fixed to avoid blinking
    text.chars()
        .enumerate()
        .map(|(i, ch)| {
            let c = if ch == ' ' { Color::Gray } else { pal[(i + off) % pal.len()] };
            Span::styled(ch.to_string(), Style::default().fg(c).add_modifier(Modifier::BOLD))
        })
        .collect()
}
