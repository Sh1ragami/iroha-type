use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{layout::{Constraint, Direction, Layout}, style::{Style, Color}, text::Span, widgets::{Block, Borders, Paragraph}, Terminal, Frame};

use crate::ui;
use crate::engine;
use crate::store;
use crate::util;

use engine::game::{Game, GameConfig, WordEntry};
use store::json::{ScoreBook, ScoreRecord};
use util::config::AppConfig;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen { Top, Play, Result, Ranking, Details, Settings, Help }

pub struct App {
    pub screen: Screen,
    pub quit: bool,
    pub game: Option<Game>,
    pub scorebook: ScoreBook,
    pub last_result: Option<ScoreRecord>,
    pub cfg: AppConfig,
    theme: Theme,
    pub anim_tick: u64,
    pub words: Vec<WordEntry>,
    pub replay: Option<ReplayState>,
}

#[derive(Clone, Copy)]
pub struct Theme {
    pub fg: Color,
    pub bg: Color,
    pub accent: Color,
    pub good: Color,
    pub bad: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self { fg: Color::White, bg: Color::Black, accent: Color::Cyan, good: Color::Green, bad: Color::Red }
    }
}

pub fn run(terminal: &mut Terminal<ratatui::prelude::CrosstermBackend<std::io::Stdout>>) -> Result<()> {
    let data_dir = std::path::PathBuf::from("data");
    let words_path = data_dir.join("words").join("basic_common.json");
    let rules_path = data_dir.join("rules").join("romaji.yaml");

    let cfg = AppConfig::load_or_default()?;
    let words: Vec<WordEntry> = engine::game::load_words_json(&words_path)?;
    let scorebook = store::json::ScoreBook::load_or_default()?;

    let mut app = App {
        screen: Screen::Top,
        quit: false,
        game: None,
        scorebook,
        last_result: None,
        cfg,
        theme: Theme::default(),
        anim_tick: 0,
        words: words.clone(),
        replay: None,
    };

    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(16);

    loop {
        terminal.draw(|f| draw(f, &mut app))?;
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                handle_key(&mut app, key, &words, &rules_path)?;
            }
        }
        if last_tick.elapsed() >= tick_rate {
            if let Some(g) = &mut app.game { g.on_tick(); }
            // replay time update
            if matches!(app.screen, Screen::Details) {
                if let (Some(rep), Some(rec)) = (&mut app.replay, &app.last_result) {
                    if rep.playing {
                        rep.time += tick_rate.as_secs_f64() * rep.speed;
                        if let Some(evs) = &rec.replay {
                            while rep.ev_idx + 1 < evs.len() && evs[rep.ev_idx + 1].t <= rep.time { rep.ev_idx += 1; }
                        }
                    }
                }
            }
            app.anim_tick = app.anim_tick.wrapping_add(1);
            last_tick = Instant::now();
        }
        if app.quit { break; }
    }
    Ok(())
}

fn handle_key(app: &mut App, key: KeyEvent, words: &Vec<WordEntry>, rules_path: &std::path::Path) -> Result<()> {
    match app.screen {
        Screen::Top => {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => app.quit = true,
                KeyCode::Char('g') | KeyCode::Enter => {
                    let mut gc = GameConfig::default();
                    gc.loss_ms_per_miss = app.cfg.loss_ms_per_miss;
                    // 設定からモードを反映
                    gc.fixed_chars = app.cfg.fixed_chars;
                    gc.target_chars = app.cfg.target_chars as usize;
                    gc.time_limit_sec = f64::INFINITY;
                    gc.max_words = usize::MAX; // 固定文字数モードでは周回できるよう制限なし
                    let mut g = Game::new(gc, words.clone(), rules_path)?;
                    g.start();
                    app.game = Some(g);
                    app.screen = Screen::Play;
                }
                KeyCode::Char('r') => app.screen = Screen::Ranking,
                KeyCode::Char('s') => app.screen = Screen::Settings,
                _ => {}
            }
        }
        Screen::Play => {
            if let Some(g) = &mut app.game {
                let finished = g.handle_key(key)?;
                if finished {
                    let record = g.finish_record();
                    app.scorebook.update_with(record.clone());
                    app.scorebook.save()?;
                    app.last_result = Some(record);
                    // プレイ終了後はランキング画面へ直接遷移
                    app.screen = Screen::Ranking;
                }
            }
        }
        Screen::Result => {
            match key.code {
                KeyCode::Esc => app.screen = Screen::Top,
                KeyCode::Enter => app.screen = Screen::Ranking,
                _ => {}
            }
        }
        Screen::Ranking => {
            match key.code {
                KeyCode::Esc => app.screen = Screen::Top,
                KeyCode::Enter => { app.screen = Screen::Details; if app.replay.is_none() { app.replay = Some(ReplayState::default()); } },
                _ => {}
            }
        }
        Screen::Details => {
            match key.code {
                KeyCode::Esc => app.screen = Screen::Top,
                KeyCode::Char('r') => app.screen = Screen::Ranking,
                KeyCode::Char(' ') => { if let Some(rep)=&mut app.replay { rep.playing = !rep.playing; } },
                KeyCode::Left => { if let Some(rep)=&mut app.replay { rep.ev_idx = rep.ev_idx.saturating_sub(1); rep.playing=false; } },
                KeyCode::Right => { if let Some(rep)=&mut app.replay { rep.ev_idx = rep.ev_idx.saturating_add(1); rep.playing=false; } },
                KeyCode::Char('+') => { if let Some(rep)=&mut app.replay { rep.speed = (rep.speed*1.25).min(4.0);} },
                KeyCode::Char('-') => { if let Some(rep)=&mut app.replay { rep.speed = (rep.speed/1.25).max(0.25);} },
                _ => {}
            }
        }
        Screen::Settings => {
            match key.code {
                KeyCode::Esc => app.screen = Screen::Top,
                KeyCode::Char('+') => { app.cfg.loss_ms_per_miss = (app.cfg.loss_ms_per_miss + 10).min(2000); app.cfg.save()?; }
                KeyCode::Char('-') => { app.cfg.loss_ms_per_miss = app.cfg.loss_ms_per_miss.saturating_sub(10); app.cfg.save()?; }
                KeyCode::Left => { app.cfg.stage_w = app.cfg.stage_w.saturating_sub(2).max(40); app.cfg.save()?; }
                KeyCode::Right => { app.cfg.stage_w = (app.cfg.stage_w + 2).min(160); app.cfg.save()?; }
                KeyCode::Up => { app.cfg.stage_h = (app.cfg.stage_h + 1).min(60); app.cfg.save()?; }
                KeyCode::Down => { app.cfg.stage_h = app.cfg.stage_h.saturating_sub(1).max(12); app.cfg.save()?; }
                KeyCode::Char('f') | KeyCode::Char('F') => { app.cfg.fixed_chars = !app.cfg.fixed_chars; app.cfg.save()?; }
                KeyCode::Char(']') => { app.cfg.target_chars = (app.cfg.target_chars + 50).min(2000); app.cfg.save()?; }
                KeyCode::Char('[') => { app.cfg.target_chars = app.cfg.target_chars.saturating_sub(50).max(50); app.cfg.save()?; }
                _ => {}
            }
        }
        Screen::Help => { if key.code == KeyCode::Esc { app.screen = Screen::Top; } }
    }
    Ok(())
}

fn draw(f: &mut Frame, app: &mut App) {
    match app.screen {
        Screen::Top => ui::top::draw(f, app),
        Screen::Play => ui::play::draw(f, app),
        Screen::Result => ui::result::draw(f, app),
        Screen::Ranking => ui::ranking::draw(f, app),
        Screen::Details => ui::details::draw(f, app),
        Screen::Settings => ui::settings::draw(f, app),
        Screen::Help => {
            let layout = Layout::default().direction(Direction::Vertical).constraints([
                Constraint::Percentage(100)
            ]).split(f.size());
            let p = Paragraph::new("Help: [G]O!, [R]EADY, [S]ettings, [R]anking, ESC=Back, q=Quit").block(Block::default().borders(Borders::ALL).title("Help"));
            f.render_widget(p, layout[0]);
        }
    }
}

pub use ui::*;

#[derive(Debug, Clone)]
pub struct ReplayState { pub ev_idx: usize, pub playing: bool, pub speed: f64, pub time: f64 }
impl Default for ReplayState { fn default()->Self{ Self{ ev_idx:0, playing:true, speed:1.0, time:0.0 } }}
