use std::{fs, path::PathBuf};
use anyhow::Result;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub loss_ms_per_miss: u64,
    pub theme: String,
    #[serde(default = "default_app_name")] pub app_name: String,
    #[serde(default = "default_stage_w")] pub stage_w: u16,
    #[serde(default = "default_stage_h")] pub stage_h: u16,
    #[serde(default = "default_fixed_chars")] pub fixed_chars: bool,
    #[serde(default = "default_target_chars")] pub target_chars: u32,
}

impl Default for AppConfig {
    fn default() -> Self { Self{ loss_ms_per_miss: 200, theme: "default".into(), app_name: default_app_name(), stage_w: default_stage_w(), stage_h: default_stage_h(), fixed_chars: default_fixed_chars(), target_chars: default_target_chars() } }
}

impl AppConfig {
    pub fn path() -> PathBuf { PathBuf::from("data/config.json") }
    pub fn load_or_default() -> Result<Self> {
        let p = Self::path();
        if let Ok(s) = fs::read_to_string(&p) { Ok(serde_json::from_str(&s).unwrap_or_default()) } else { Ok(Self::default()) }
    }
    pub fn save(&self) -> Result<()> { fs::create_dir_all("data")?; fs::write(Self::path(), serde_json::to_string_pretty(self)?)?; Ok(()) }
}

fn default_stage_w() -> u16 { 88 }
fn default_stage_h() -> u16 { 28 }
fn default_fixed_chars() -> bool { true }
fn default_target_chars() -> u32 { 400 }
fn default_app_name() -> String { "IrohaType".into() }
