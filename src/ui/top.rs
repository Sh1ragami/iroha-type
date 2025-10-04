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
            Constraint::Length(2),  // top spacer
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

    // spacer row
    f.render_widget(Paragraph::new(""), v[1]);

    let center = v[2];
    let mut body: Vec<Line> = ascii_title_lines(&app.cfg.app_name);
    body.push(Line::from(""));
    body.push(Line::from("Japanese Typing TUI").style(Style::default().fg(Color::Gray)));
    let title = Paragraph::new(body).alignment(Alignment::Center);
    f.render_widget(title, center);

    let foot = if let Some(r) = &app.last_result {
        Paragraph::new(Line::from(format!(
            "Last  {:.3}s  Miss {}  Rank {}  Top {:.1} WPM",
            r.time_sec, r.miss, r.rank, r.wpm_top
        ))).alignment(Alignment::Center)
    } else {
        Paragraph::new(Line::from("Press G to begin.")).alignment(Alignment::Center)
    };
    f.render_widget(foot, v[3]);
}

fn ascii_title_lines(name: &str) -> Vec<Line<'static>> {
    // 5x5 block font (soft two-tone, wider tracking)
    let text = name.to_uppercase();
    let rows = 5usize;
    let mut out: Vec<String> = vec![String::new(); rows];
    for ch in text.chars() {
        let pat: [&str;5] = match ch {
            'A' => [" ███ ", "█   █", "█████", "█   █", "█   █"],
            'E' => ["█████", "█    ", "████ ", "█    ", "█████"],
            'H' => ["█   █", "█   █", "█████", "█   █", "█   █"],
            'I' => ["█████", "  █  ", "  █  ", "  █  ", "█████"],
            'O' => ["█████", "█   █", "█   █", "█   █", "█████"],
            'P' => ["████ ", "█   █", "████ ", "█    ", "█    "],
            'R' => ["████ ", "█   █", "████ ", "█  █ ", "█   █"],
            'T' => ["█████", "  █  ", "  █  ", "  █  ", "  █  "],
            'Y' => ["█   █", " █ █ ", "  █  ", "  █  ", "  █  "],
            ' ' => ["     ", "     ", "     ", "     ", "     "],
            _ => ["     ", "     ", "     ", "     ", "     "],
        };
        for r in 0..rows {
            out[r].push_str(pat[r]);
            out[r].push(' ');
            out[r].push(' '); // extra tracking for美観
        }
    }
    let row_colors = [Color::LightCyan, Color::White, Color::LightBlue, Color::White, Color::Gray];
    out.into_iter()
        .enumerate()
        .map(|(i, s)| Line::from(Span::styled(s, Style::default().fg(row_colors[i]).add_modifier(Modifier::BOLD))))
        .collect()
}
