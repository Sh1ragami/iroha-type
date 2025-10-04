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
    #[serde(skip_serializing_if = "Option::is_none")] pub speed_series: Option<Vec<(f64,f64)>>,
    #[serde(skip_serializing_if = "Option::is_none")] pub word_display: Option<String>,
}

impl ScoreBook {
    pub fn path() -> PathBuf { PathBuf::from("data/scores.json") }
    pub fn load_or_default() -> Result<Self> {
        let p = Self::path();
        if let Ok(s) = fs::read_to_string(&p) { Ok(serde_json::from_str(&s)?) } else { Ok(Self::default()) }
    }
    pub fn save(&self) -> Result<()> { fs::create_dir_all("data")?; fs::write(Self::path(), serde_json::to_string_pretty(self)?)?; Ok(()) }
    pub fn update_with(&mut self, rec: ScoreRecord) {
        self.top.push(rec.clone());
        self.top.sort_by(|a,b| b.wpm_top.total_cmp(&a.wpm_top));
        self.top.truncate(15);
        // Update Lap10 from splits
        for s in &rec.splits {
            let mut laprec = ScoreRecord{ time_sec: s.sec, word_display: Some(s.word.clone()), ..Default::default() };
            self.lap.push(laprec);
        }
        self.lap.sort_by(|a,b| a.time_sec.partial_cmp(&b.time_sec).unwrap());
        self.lap.truncate(10);
    }
}

