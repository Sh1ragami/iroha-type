use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use serde::{Deserialize, Serialize};

use super::romaji::{RomajiMatcher, RomajiRules};
use super::stats::compute_wpm_stats;
use rand::seq::SliceRandom;
use crate::store::json::KeyEv;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordEntry { pub jp: String, pub romas: Vec<String> }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WordsFile { pub title: String, pub version: u32, pub entries: Vec<WordEntry> }

#[derive(Debug, Clone)]
pub struct Split { pub word: String, pub sec: f64, pub miss: u32, pub keystrokes: u32 }

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub time_limit_sec: f64,
    pub max_words: usize,
    pub loss_ms_per_miss: u64,
    pub fixed_chars: bool,
    pub target_chars: usize,
}
impl Default for GameConfig {
    fn default() -> Self { Self { time_limit_sec: 60.0, max_words: 50, loss_ms_per_miss: 200, fixed_chars: false, target_chars: 0 } }
}

pub fn load_words_json(path: &Path) -> Result<Vec<WordEntry>> {
    let data = fs::read_to_string(path)?;
    let wf: WordsFile = serde_json::from_str(&data)?;
    Ok(wf.entries)
}

pub fn load_words_from_str(data: &str) -> Result<Vec<WordEntry>> {
    let wf: WordsFile = serde_json::from_str(data)?;
    Ok(wf.entries)
}

fn min_roma_len(e: &WordEntry) -> usize {
    e.romas.iter().map(|s| s.len()).min().unwrap_or(0)
}

fn build_session_words(all: &[WordEntry], _rules: &RomajiRules, target_chars: usize) -> Vec<WordEntry> {
    if all.is_empty() { return vec![]; }
    let mut rng = rand::thread_rng();
    let mut pool: Vec<WordEntry> = all.to_vec();
    pool.shuffle(&mut rng);
    if target_chars == 0 { return pool; }
    let mut out: Vec<WordEntry> = Vec::new();
    let mut sum = 0usize;
    // cycle through shuffled pool until reaching/exceeding target_chars
    for e in pool.iter().cycle() {
        let len = min_roma_len(e);
        out.push(e.clone());
        sum += len;
        if sum >= target_chars || out.len() > all.len()*3 { break; }
    }
    out
}

pub struct Game {
    pub words: Vec<WordEntry>,
    idx: usize,
    typed: String,
    correct_keystrokes: u32,
    miss: u32,
    pub splits: Vec<Split>,
    started_at: Option<Instant>,
    // Freeze end time to stop timer after finish
    ended_at: Option<Instant>,
    word_start: Option<Instant>,
    finished: bool,
    aborted: bool,
    rules: RomajiRules,
    matcher: Option<RomajiMatcher>,
    speed_series: Vec<(f64,f64)>,
    cfg: GameConfig,
    last_miss_char: Option<char>,
    replay: Vec<KeyEv>,
    // Stores the romaji variant actually typed for each word (if completed)
    display_romas: Vec<Option<String>>,
}

impl Game {
    pub fn new(cfg: GameConfig, words: Vec<WordEntry>, rules_path: &Path) -> Result<Self> {
        let rules = RomajiRules::from_yaml_file(rules_path)?;
        let mut words_sel = if cfg.fixed_chars && cfg.target_chars > 0 { build_session_words(&words, &rules, cfg.target_chars) } else { words };
        if !words_sel.is_empty() && !cfg.fixed_chars { words_sel.truncate(cfg.max_words.min(words_sel.len())); }
        let dr_len = words_sel.len();
        Ok(Self{
            words: words_sel,
            idx: 0,
            typed: String::new(),
            correct_keystrokes: 0,
            miss: 0,
            splits: vec![],
            started_at: None,
            ended_at: None,
            word_start: None,
            finished: false,
            aborted: false,
            rules,
            matcher: None,
            speed_series: vec![],
            cfg,
            last_miss_char: None,
            replay: vec![],
            display_romas: vec![None; dr_len],
        })
    }

    pub fn new_with_rules(cfg: GameConfig, words: Vec<WordEntry>, rules: RomajiRules) -> Result<Self> {
        let mut words_sel = if cfg.fixed_chars && cfg.target_chars > 0 { build_session_words(&words, &rules, cfg.target_chars) } else { words };
        if !words_sel.is_empty() && !cfg.fixed_chars { words_sel.truncate(cfg.max_words.min(words_sel.len())); }
        let dr_len = words_sel.len();
        Ok(Self{
            words: words_sel,
            idx: 0,
            typed: String::new(),
            correct_keystrokes: 0,
            miss: 0,
            splits: vec![],
            started_at: None,
            ended_at: None,
            word_start: None,
            finished: false,
            aborted: false,
            rules,
            matcher: None,
            speed_series: vec![],
            cfg,
            last_miss_char: None,
            replay: vec![],
            display_romas: vec![None; dr_len],
        })
    }

    pub fn start(&mut self) {
        // 計測は最初の打鍵で開始するため、ここでは開始しない
        self.started_at = None;
        self.word_start = None;
        if self.display_romas.len() != self.words.len() { self.display_romas = vec![None; self.words.len()]; }
        if let Some(w) = self.words.get(self.idx) {
            self.matcher = Some(RomajiMatcher::new(&w.jp, &w.romas, &self.rules));
        }
    }

    // 明示的に計測を開始（カウントダウン後に呼び出し）
    pub fn begin_now(&mut self) {
        if self.started_at.is_none() {
            let now = Instant::now();
            self.started_at = Some(now);
            self.word_start = Some(now);
        }
    }

    pub fn on_tick(&mut self) {
        if self.finished { return; }
        let t = self.elapsed_secs();
        let cps = if t>0.0 { self.correct_keystrokes as f64 / t } else { 0.0 };
        self.speed_series.push((t,cps));
        let fixed_done = self.cfg.fixed_chars && (self.correct_keystrokes as usize) >= self.cfg.target_chars;
        let all_words_done = self.idx >= self.words.len() || self.idx >= self.cfg.max_words;
        if (self.cfg.fixed_chars && fixed_done) || (!self.cfg.fixed_chars && all_words_done) { self.finish(); }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        if self.finished { return Ok(true); }
        match key.code {
            KeyCode::Esc => { self.aborted = true; self.finish(); return Ok(true); }
            KeyCode::Char(ch) => {
                let c = ch.to_ascii_lowercase();
                // 最初の打鍵で計測開始
                if self.started_at.is_none() {
                    self.started_at = Some(Instant::now());
                    self.word_start = Some(Instant::now());
                }
                if let Some(m) = &mut self.matcher { 
                    match m.input_char(c) {
                        super::romaji::InputResult::Correct => {
                            self.typed.push(c); self.correct_keystrokes+=1; self.last_miss_char=None; self.push_ev(c, true);
                            if self.cfg.fixed_chars && (self.correct_keystrokes as usize) >= self.cfg.target_chars { self.finish(); return Ok(true); }
                        },
                        super::romaji::InputResult::Miss => { self.miss+=1; self.penalize_time(); self.last_miss_char = Some(c); self.push_ev(c, false); },
                        super::romaji::InputResult::Complete => {
                            self.typed.push(c); self.correct_keystrokes+=1; self.last_miss_char=None; self.push_ev(c, true);
                            if self.cfg.fixed_chars && (self.correct_keystrokes as usize) >= self.cfg.target_chars { self.finish(); return Ok(true); }
                            self.finish_word();
                        },
                        super::romaji::InputResult::Noop => {}
                    }
                }
            }
            _ => {}
        }
        Ok(self.finished)
    }

    fn penalize_time(&mut self) {
        // Add a phantom delay by shifting timers backward
        if let Some(ws) = &mut self.word_start {
            *ws -= Duration::from_millis(self.cfg.loss_ms_per_miss);
        }
        if let Some(st) = &mut self.started_at {
            *st -= Duration::from_millis(self.cfg.loss_ms_per_miss);
        }
    }

    fn finish_word(&mut self) {
        if let Some(ws) = self.word_start.take() {
            let sec = ws.elapsed().as_secs_f64();
            let word = self.words[self.idx].jp.clone();
            let miss = self.matcher.as_ref().map(|m| m.miss_count).unwrap_or(0);
            let ks = self.matcher.as_ref().map(|m| {
                let def = self.words[self.idx].romas.get(0).cloned().unwrap_or_default();
                m.display_with_default(&def).len() as u32
            }).unwrap_or(0);
            self.splits.push(Split{word, sec, miss, keystrokes: ks});
        }
        // Record the actual variant used for display if available
        if let Some(m) = &self.matcher {
            let used = if m.typed.is_empty() {
                let def = self.words[self.idx].romas.get(0).cloned().unwrap_or_default();
                m.display_with_default(&def)
            } else { m.typed.clone() };
            if self.idx < self.display_romas.len() { self.display_romas[self.idx] = Some(used); }
        }
        self.idx += 1;
        self.typed.clear();
        self.word_start = Some(Instant::now());
        if self.idx < self.words.len() && self.idx < self.cfg.max_words {
            let w = &self.words[self.idx];
            self.matcher = Some(RomajiMatcher::new(&w.jp, &w.romas, &self.rules));
        } else {
            // fixed_charsでも周回せず終了（表示は目標打数を超える最後の語まで）
            self.finish();
        }
    }

    pub fn finish_record(&self) -> crate::store::json::ScoreRecord {
        let time_sec = self.elapsed_secs();
        let timeloss_sec = (self.cfg.loss_ms_per_miss as f64 * self.miss as f64)/1000.0;
        let (wpm_top, wpm_worst) = compute_wpm_stats(&self.splits);
        crate::store::json::ScoreRecord {
            mode: "basic_common".into(),
            datetime: chrono::Local::now().to_rfc3339(),
            time_sec,
            miss: self.miss,
            timeloss_sec,
            splits: self.splits.iter().map(|s| crate::store::json::SplitRec { word: s.word.clone(), sec: s.sec, miss: s.miss }).collect(),
            wpm_top, wpm_worst,
            rank: super::level::estimate_rank(self.avg_cps()).to_string(),
            memo: None,
            speed_series: Some(self.speed_series.clone()),
            word_display: None,
            replay: if self.replay.is_empty() { None } else { Some(self.replay.clone()) },
        }
    }

    pub fn elapsed_secs(&self) -> f64 {
        match (self.started_at, self.ended_at) {
            (Some(start), Some(end)) => end.saturating_duration_since(start).as_secs_f64(),
            (Some(start), None) => start.elapsed().as_secs_f64(),
            _ => 0.0,
        }
    }
    pub fn time_left_secs(&self) -> f64 { (self.cfg.time_limit_sec - self.elapsed_secs()).max(0.0) }
    pub fn current_level(&self) -> String { super::level::estimate_rank(self.avg_cps()).to_string() }
    pub fn miss(&self) -> u32 { self.miss }
    pub fn typed(&self) -> &str { &self.typed }
    pub fn progress_ratio(&self) -> f64 {
        if self.cfg.fixed_chars {
            if self.cfg.target_chars == 0 { return 0.0; }
            (self.correct_keystrokes as f64 / self.cfg.target_chars as f64).clamp(0.0, 1.0)
        } else {
            let denom = (self.words.len().min(self.cfg.max_words)).max(1);
            (self.idx as f64 / denom as f64).clamp(0.0, 1.0)
        }
    }
    pub fn current_line_display(&self) -> String {
        let start = self.idx.saturating_sub(3);
        let end = (self.idx+4).min(self.words.len());
        let mut s = String::new();
        for i in start..end {
            if i==self.idx { s.push('['); }
            s.push_str(&self.words[i].jp);
            if i==self.idx { s.push(']'); }
            s.push(' ');
        }
        s
    }
    pub fn current_roma_line(&self) -> String {
        if let Some(w) = self.words.get(self.idx) {
            if let Some(m) = &self.matcher {
                let def = w.romas.get(0).cloned().unwrap_or_default();
                return m.display_with_default(&def);
            }
            w.romas.get(0).cloned().unwrap_or_default()
        } else { String::new() }
    }

    // Return the romaji display for a given word index reflecting typed method
    pub fn roma_for_index(&self, i: usize) -> String {
        if i >= self.words.len() { return String::new(); }
        if i < self.idx {
            if let Some(Some(s)) = self.display_romas.get(i) { return s.clone(); }
            return self.words[i].romas.get(0).cloned().unwrap_or_default();
        } else if i == self.idx {
            return self.current_roma_line();
        } else {
            return self.words[i].romas.get(0).cloned().unwrap_or_default();
        }
    }
    pub fn current_jp_progress(&self) -> (String, Option<char>, String) {
        if let Some(w) = self.words.get(self.idx) {
            let jp = &w.jp;
            let roma = self.current_roma_line();
            let total = roma.len().max(1);
            let typed = self.current_typed_len().min(total);
            let chars: Vec<char> = jp.chars().collect();
            let n = chars.len().max(1);
            let mut pos = ((typed as f64 / total as f64) * n as f64).floor() as usize;
            if pos >= n { pos = n.saturating_sub(1); }
            let done: String = if pos>0 { chars[..pos].iter().collect() } else { String::new() };
            let next = if typed >= total && n>0 { None } else { Some(chars[pos]) };
            let rest: String = if typed >= total { String::new() } else { chars[pos+1..].iter().collect() };
            (done, next, rest)
        } else { (String::new(), None, String::new()) }
    }
    pub fn split_table_text(&self) -> String {
        let mut s = String::new();
        for (i, sp) in self.splits.iter().enumerate() {
            s.push_str(&format!("{:>2} {}  {:>6.3}s  miss {}
", i+1, sp.word, sp.sec, sp.miss));
        }
        s
    }
    fn avg_cps(&self) -> f64 { 
        let t = self.elapsed_secs();
        if t>0.0 { self.correct_keystrokes as f64 / t } else {0.0}
    }
    pub fn current_index(&self) -> usize { self.idx }
    pub fn words_len(&self) -> usize { self.words.len() }
    pub fn speed_points(&self) -> &[(f64,f64)] { &self.speed_series }
    pub fn current_typed_len(&self) -> usize { self.typed.len() }
    pub fn last_miss_char(&self) -> Option<char> { self.last_miss_char }
    pub fn current_typed_total(&self) -> u32 { self.correct_keystrokes }
    pub fn aborted(&self) -> bool { self.aborted }

    fn push_ev(&mut self, c: char, ok: bool) {
        let t = self.elapsed_secs();
        self.replay.push(KeyEv{ t, c: c.to_string(), ok, w: self.idx });
    }

    // mark game as finished and freeze timer
    fn finish(&mut self) {
        if !self.finished { self.finished = true; }
        if self.started_at.is_some() && self.ended_at.is_none() {
            self.ended_at = Some(Instant::now());
        }
    }
}
