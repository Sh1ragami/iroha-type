use std::{fs, path::PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScoreBook { pub top: Vec<ScoreRecord>, pub lap: Vec<ScoreRecord> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitRec { pub word: String, pub sec: f64, pub miss: u32 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScoreRecord {
    pub mode: String,
    pub datetime: String,
    pub time_sec: f64,
    pub miss: u32,
    pub timeloss_sec: f64,
    pub splits: Vec<SplitRec>,
    pub wpm_top: f64,
    pub wpm_worst: f64,
    pub rank: String,
    #[serde(skip_serializing_if = "Option::is_none")] pub memo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub speed_series: Option<Vec<(f64,f64)>>,
    #[serde(skip_serializing_if = "Option::is_none")] pub word_display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub replay: Option<Vec<KeyEv>>, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEv { pub t: f64, pub c: String, pub ok: bool, pub w: usize }

impl ScoreBook {
    pub fn path() -> PathBuf { PathBuf::from("data/scores.json") }
    pub fn load_or_default() -> Result<Self> {
        let p = Self::path();
        if let Ok(s) = fs::read_to_string(&p) { Ok(serde_json::from_str(&s)?) } else { Ok(Self::default()) }
    }
    pub fn save(&self) -> Result<()> { fs::create_dir_all("data")?; fs::write(Self::path(), serde_json::to_string_pretty(self)?)?; Ok(()) }

    /// Insert a record, keep Top-100 by fastest time, and return:
    /// (rank position 1-based if within Top-100, is_personal_best)
    pub fn insert_and_rank(&mut self, rec: ScoreRecord) -> (Option<usize>, bool) {
        let prev_best_time = self.top.iter().map(|r| r.time_sec).fold(f64::INFINITY, |a,b| a.min(b));
        self.top.push(rec.clone());
        // Time-centric: lower time is better
        self.top.sort_by(|a,b| a.time_sec.partial_cmp(&b.time_sec).unwrap());
        // find index by unique datetime
        let idx = self.top.iter().position(|r| r.datetime == rec.datetime);
        // Keep Top-100
        if self.top.len() > 100 { self.top.truncate(100); }
        let rank_in = idx.and_then(|i| if i < 100 { Some(i+1) } else { None });
        let is_new_pb = if prev_best_time.is_finite() { rec.time_sec < prev_best_time } else { true };

        // Update Lap10 from splits (still by fastest laps)
        for s in &rec.splits {
            let laprec = ScoreRecord{ time_sec: s.sec, word_display: Some(s.word.clone()), ..Default::default() };
            self.lap.push(laprec);
        }
        self.lap.sort_by(|a,b| a.time_sec.partial_cmp(&b.time_sec).unwrap());
        self.lap.truncate(10);

        (rank_in, is_new_pb)
    }

    /// Set memo for a record identified by datetime. Returns true if updated.
    pub fn set_memo_by_datetime(&mut self, datetime: &str, memo: String) -> bool {
        if let Some(r) = self.top.iter_mut().find(|r| r.datetime == datetime) {
            r.memo = Some(memo);
            true
        } else { false }
    }
}
