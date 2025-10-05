use ratatui::{prelude::*, widgets::*};
use crate::app::App;

pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.size();
    // Centered stage for play screen (inspired by TypeWell window size)
    let stage = super::centered(area, app.cfg.stage_w.into(), app.cfg.stage_h.into());
    f.render_widget(Clear, area);
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // header (GO/READY/タイム)
            Constraint::Percentage(38), // JP grid
            Constraint::Length(1),      // progress blocks
            Constraint::Length(1),      // info row
            Constraint::Percentage(40), // ROMA + right sidebar
            Constraint::Length(1),      // gauge
        ])
        .split(stage);

    // ヘッダー
    if let Some(g) = &app.game {
        let header_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(16),
                Constraint::Min(10),
                Constraint::Length(18),
            ])
            .split(v[0]);

        let btns = Paragraph::new(Line::from(vec![
            Span::styled(" GO! ", Style::default().fg(Color::White).bg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(" READY ", Style::default().fg(Color::Black).bg(Color::Gray)),
        ]));
        f.render_widget(btns, header_cols[0]);

        let title = Paragraph::new(Line::from(vec![
            Span::styled("【 基本常用語 】", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ])).alignment(Alignment::Center);
        f.render_widget(title, header_cols[1]);

        let tbox = Paragraph::new(Line::from(vec![
            Span::raw("タイム: "),
            Span::styled(format!("[{:.1}]", g.elapsed_secs()), Style::default().fg(Color::Yellow)),
            Span::raw("  Keys "),
            Span::styled(format!("{:>3}/400", g.current_typed_total()), Style::default().fg(Color::LightGreen)),
        ])).alignment(Alignment::Right);
        f.render_widget(tbox, header_cols[2]);

        // JP グリッド
        let jp_line = Paragraph::new(Line::from(jp_spans_grid(g)))
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title("かな/漢字"));
        f.render_widget(jp_line, v[1]);

        // プログレスブロック
        let blocks = progress_blocks(g.progress_ratio(), 28);
        f.render_widget(Paragraph::new(blocks), v[2]);

        // 情報行
        let info = Line::from(vec![
            Span::styled("目標=400打 ", Style::default().fg(Color::Red)),
            Span::raw("  "),
            Span::styled(format!("レベル {} ", g.current_level()), Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::styled(format!("ミス {}", g.miss()), Style::default().fg(Color::Red)),
        ]);
        f.render_widget(Paragraph::new(info), v[3]);

        // ROMA line: 完了=緑、現在=typed部緑+ミス赤、未了=白
        let mut roma_spans: Vec<Span> = Vec::new();
        let idx = g.current_index();
        for i in 0..g.words_len() {
            let roma = g.roma_for_index(i);
            if i < idx {
                roma_spans.push(Span::styled(roma, Style::default().fg(Color::Green)));
            } else if i == idx {
                let typed_len = g.current_typed_len();
                let (a,b) = roma.split_at(typed_len.min(roma.len()));
                if !a.is_empty() {
                    roma_spans.push(Span::styled(a.to_string(), Style::default().fg(Color::Green)));
                }
                if !b.is_empty() {
                    if g.last_miss_char().is_some() {
                        let mut chs = b.chars();
                        if let Some(expected) = chs.next() {
                            roma_spans.push(Span::styled(expected.to_string(), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
                        }
                        let rest: String = chs.collect();
                        if !rest.is_empty() { roma_spans.push(Span::raw(rest)); }
                    } else {
                        roma_spans.push(Span::raw(b.to_string()));
                    }
                }
            } else {
                roma_spans.push(Span::raw(roma));
            }
            if i + 1 < g.words_len() { roma_spans.push(Span::raw(" ")); }
        }
        let main_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
            .split(v[4]);
        // ROMA only
        let roma_line = Paragraph::new(Line::from(roma_spans))
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title("ローマ字"));
        f.render_widget(roma_line, main_cols[0]);

        // 下部: スプリットテーブル（最大限エリアを埋める）
        let rows: Vec<Row> = g
            .splits
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let mut row = Row::new(vec![
                    Cell::from(format!("{:>2}", i + 1)),
                    Cell::from(format!("{:>6.3}", s.sec)),
                ]);
                if s.miss > 0 { row = row.style(Style::default().fg(Color::Red)); }
                row
            })
            .collect();
        let table = Table::new(rows, [Constraint::Length(3), Constraint::Length(8)])
            .block(Block::default().borders(Borders::ALL).title("Lap"))
            .header(Row::new(vec!["#", "秒"]).style(Style::default().fg(Color::Yellow)))
            .column_spacing(1);
        f.render_widget(table, main_cols[1]);

        // 下: 進行ゲージ 1行
        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Cyan))
            .ratio(g.progress_ratio());
        f.render_widget(gauge, v[5]);
    }

    // Overlay: 終了時ダイアログ（新記録/順位/名前入力）
    if let Some(prompt) = &app.rec_prompt {
        let dlg_w = 62u16; let dlg_h = if prompt.rank_in_top.is_some() { 8 } else { 6 };
        let dlg = super::centered(stage, dlg_w, dlg_h);
        // 透過風に上書き
        f.render_widget(Clear, dlg);
        let title = if prompt.is_new {
            Span::styled(" 新記録樹立！ ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(" 記録 ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        };
        let blk = Block::default().borders(Borders::ALL).title(title);
        f.render_widget(blk, dlg);

        let inner = Rect{ x: dlg.x+1, y: dlg.y+1, width: dlg.width-2, height: dlg.height-2 };
        let lines = if let (Some(rk), Some(rec)) = (prompt.rank_in_top, &app.last_result) {
            vec![
                Line::from(Span::styled("おめでとうございます。", Style::default().fg(Color::Cyan))),
                Line::from(Span::styled(format!("記録: {:>.3} 秒", rec.time_sec), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                Line::from(Span::styled(format!("TOP100入り（{}位）", rk), Style::default().fg(Color::Magenta))),
                Line::from("[Enter] ランキングへ   [ESC] トップへ"),
            ]
        } else if let Some(rec) = &app.last_result {
            vec![
                Line::from(Span::styled(format!("記録: {:>.3} 秒", rec.time_sec), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                Line::from(Span::styled("今回はTOP100に入りませんでした。", Style::default().fg(Color::Gray))),
                Line::from("[Enter] ランキングへ   [ESC] トップへ"),
            ]
        } else { vec![Line::from("[Enter] ランキングへ   [ESC] トップへ")] };
        let para = Paragraph::new(lines);
        f.render_widget(para, Rect{ x: inner.x, y: inner.y, width: inner.width, height: inner.height.saturating_sub(2) });

    }
}

// (unused) truncate helper was removed to silence warnings

fn progress_blocks(ratio: f64, cells: usize) -> Line<'static> {
    let filled = ((ratio.clamp(0.0, 1.0)) * cells as f64).round() as usize;
    let mut spans: Vec<Span> = Vec::with_capacity(cells);
    for i in 0..cells {
        if i < filled { spans.push(Span::styled("■", Style::default().fg(Color::Yellow))); }
        else { spans.push(Span::styled("□", Style::default().fg(Color::Gray))); }
    }
    Line::from(spans)
}

fn jp_spans_grid(g: &crate::engine::game::Game) -> Vec<Span<'static>> {
    let mut out: Vec<Span> = Vec::new();
    let idx = g.current_index();
    for (i, w) in g.words.iter().enumerate() {
        if i == idx {
            let roma = g.current_roma_line();
            let total = roma.len().max(1);
            let typed = g.current_typed_len().min(total);
            let chars: Vec<char> = w.jp.chars().collect();
            let n = chars.len().max(1);
            let mut pos = ((typed as f64 / total as f64) * n as f64).floor() as usize;
            if pos >= n { pos = n - 1; }
            for (j, ch) in chars.into_iter().enumerate() {
                let st = if j < pos {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else if j == pos {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                } else {
                    Style::default().fg(Color::White)
                };
                out.push(Span::styled(ch.to_string(), st));
            }
        } else {
            out.push(Span::styled(w.jp.clone(), Style::default().fg(Color::Gray)));
        }
        if i + 1 < g.words_len() { out.push(Span::raw("  ")); }
    }
    out
}
