use ratatui::{prelude::*, widgets::*};
use crate::app::App;
use crate::engine::game::WordEntry;
// use crate::engine::romaji::{RomajiMatcher, RomajiRules};

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.size();
    let stage = super::centered(area, app.cfg.stage_w.into(), app.cfg.stage_h.into());
    f.render_widget(Clear, area);
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Length(6), // jp/roma line (replay overlay)
            Constraint::Min(8),    // lists and metrics + chart
            Constraint::Length(1), // footer
        ])
        .split(stage);

    f.render_widget(Paragraph::new("[ESC] 戻る  /  [R] ランキングへ  —  記録詳細"), v[0]);

    if let Some(rec) = &app.last_result {
        // Top: romaji words line + right times + replay overlay
        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(78), Constraint::Percentage(22)])
            .split(v[1]);

        let para = Paragraph::new(Line::from(replay_line_spans(app, rec)))
            .wrap(Wrap{ trim:false })
            .block(Block::default().borders(Borders::ALL).title("Replay"));
        f.render_widget(para, top[0]);

        let mut times = String::new();
        for (i, s) in rec.splits.iter().take((top[1].height as usize).saturating_sub(2)).enumerate() {
            let mark = if s.miss>0 { "*" } else { " " };
            times.push_str(&format!("{:>2} {} {:>6.3}s\n", i+1, mark, s.sec));
        }
        let list = Paragraph::new(times).block(Block::default().borders(Borders::ALL).title("Time/word"));
        f.render_widget(list, top[1]);

        // Middle: three columns (fast, slow, metrics + chart under)
        let mid = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(34), Constraint::Percentage(34), Constraint::Percentage(32)])
            .split(v[2]);

        // Fastest (by sec)
        let mut sorted = rec.splits.clone();
        sorted.sort_by(|a,b| a.sec.partial_cmp(&b.sec).unwrap());
        let fast_lines: Vec<Line> = sorted.iter().take(7).map(|s| Line::from(format!("{:>6.3}s  {}", s.sec, s.word))).collect();
        let fast = Paragraph::new(fast_lines).block(Block::default().borders(Borders::ALL).title("速い語句"));
        f.render_widget(fast, mid[0]);

        // Slow (worst)
        let mut sorted2 = rec.splits.clone();
        sorted2.sort_by(|a,b| b.sec.partial_cmp(&a.sec).unwrap());
        let slow_lines: Vec<Line> = sorted2.iter().take(7).map(|s| Line::from(format!("{:>6.3}s  {}", s.sec, s.word))).collect();
        let slow = Paragraph::new(slow_lines).block(Block::default().borders(Borders::ALL).title("苦手語句"));
        f.render_widget(slow, mid[1]);

        // Metrics
        let avg = if !rec.splits.is_empty() { rec.splits.iter().map(|s| s.sec).sum::<f64>() / rec.splits.len() as f64 } else { 0.0 };
        let box_lines = vec![
            Line::from(format!("Time     {:>7.3}s", rec.time_sec)),
            Line::from(format!("Miss     {:>3}", rec.miss)),
            Line::from(format!("Timeloss {:>7.3}s", rec.timeloss_sec)),
            Line::from(format!("Level    {}", rec.rank)),
            Line::from(format!("Avg/word {:>7.3}s", avg)),
            Line::from(format!("Top WPM  {:>6.1}", rec.wpm_top)),
        ];
        let metrics = Paragraph::new(box_lines)
            .block(Block::default().borders(Borders::ALL).title("統計"));
        f.render_widget(metrics, mid[2]);

        // Speed chart below metrics area (reuse result chart)
        let chart_area = Rect { x: mid[2].x, y: mid[2].y + mid[2].height.saturating_sub(8), width: mid[2].width, height: 8 };
        super::chart::draw_speed_chart(f, chart_area, rec);

        // Footer
        f.render_widget(Paragraph::new("Space: 再生/停止  ←/→: 1打進む  +/-: 再生速度  Legend: *=miss, 緑=入力済, 赤=次の正解"), v[3]);
    }
}

fn replay_line_spans(app: &App, rec: &crate::store::json::ScoreRecord) -> Vec<Span<'static>> {
    let mut spans: Vec<Span> = Vec::new();
    let (ev_idx, replay_opt) = if let Some(rp) = &app.replay { (rp.ev_idx, rec.replay.as_ref()) } else { (usize::MAX, None) };
    // Build typed lengths per word up to ev_idx
    let mut typed_map: Vec<usize> = vec![0; rec.splits.len()];
    let mut cur_w = 0usize;
    let mut last_miss = false;
    if let Some(evs) = replay_opt {
        let upto = ev_idx.min(evs.len().saturating_sub(1));
        for i in 0..=upto { if evs.is_empty() { break; } let e=&evs[i]; cur_w = e.w; if e.ok { typed_map[e.w] = typed_map[e.w].saturating_add(1); } last_miss = !e.ok; }
    }
    for (i, s) in rec.splits.iter().enumerate() {
        let jp = &s.word;
        let roma_len = find_roma_len(&app.words, jp).max(1);
        let jp_chars: Vec<char> = jp.chars().collect();
        let pos = ((typed_map[i] as f64 / roma_len as f64) * jp_chars.len() as f64).floor() as usize;
        for (j, ch) in jp_chars.into_iter().enumerate() {
            let style = if i < cur_w { Style::default().fg(Color::Green) }
                else if i == cur_w {
                    if j < pos { Style::default().fg(Color::Green) }
                    else if j == pos { if last_miss { Style::default().fg(Color::Red).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::Yellow) } }
                    else { Style::default().fg(Color::White) }
                } else { Style::default().fg(Color::White) };
            spans.push(Span::styled(ch.to_string(), style));
        }
        spans.push(Span::raw(" "));
    }
    spans
}

fn find_roma_len(words: &Vec<WordEntry>, jp: &str) -> usize {
    if let Some(w) = words.iter().find(|w| w.jp == jp) { w.romas.iter().map(|s| s.len()).min().unwrap_or(1) } else { 1 }
}
