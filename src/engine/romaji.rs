use std::{collections::{HashMap, HashSet}, fs, path::Path};

use serde::Deserialize;
use anyhow::Result;

#[derive(Debug, Clone, Deserialize)]
pub struct RomajiRules {
    #[serde(default)]
    pub kana_rules: HashMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub yure: HashMap<String, Vec<String>>, // e.g., し: ["shi","si"]
    #[serde(default)]
    pub special: Option<SpecialRules>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SpecialRules {
    #[serde(default)]
    pub n_patterns: Vec<String>, // e.g., ["n'","nn","n"]
}

impl RomajiRules {
    pub fn from_yaml_file(path: &Path) -> Result<Self> {
        let s = fs::read_to_string(path)?;
        let mut r: RomajiRules = serde_yaml::from_str(&s)?;
        if r.special.is_none() { r.special = Some(SpecialRules{ n_patterns: vec!["n'".into(), "nn".into(), "n".into()] }); }
        Ok(r)
    }
}

pub enum InputResult { Correct, Miss, Complete, Noop }

pub struct RomajiMatcher {
    jp: String,
    candidates: Vec<String>,
    pub typed: String,
    pub miss_count: u32,
}

impl RomajiMatcher {
    pub fn new(jp: &str, base_romas: &[String], rules: &RomajiRules) -> Self {
        let mut set: HashSet<String> = HashSet::new();
        for b in base_romas {
            for v in expand_yure(b, rules) { set.insert(v); }
        }
        let mut candidates: Vec<String> = set.into_iter().collect();
        candidates.sort_by_key(|s| s.len());
        Self{ jp: jp.into(), candidates, typed: String::new(), miss_count: 0 }
    }

    pub fn input_char(&mut self, c: char) -> InputResult {
        if !c.is_ascii() { return InputResult::Noop; }
        let mut next = self.typed.clone(); next.push(c);
        let mut any_prefix = false;
        let mut any_exact = false;
        for cand in &self.candidates {
            if cand.starts_with(&next) { any_prefix = true; }
            if *cand == next { any_exact = true; }
        }
        if any_exact { self.typed = next; return InputResult::Complete; }
        if any_prefix { self.typed = next; return InputResult::Correct; }
        self.miss_count += 1; InputResult::Miss
    }

    pub fn example_roma(&self) -> String {
        self.candidates.first().cloned().unwrap_or_default()
    }

    pub fn best_candidate(&self) -> String {
        // Return the shortest candidate that starts with current typed prefix.
        let pref = &self.typed;
        if pref.is_empty() { return self.example_roma(); }
        let mut best: Option<&String> = None;
        for c in &self.candidates {
            if c.starts_with(pref) {
                if let Some(cur) = &best {
                    if c.len() < cur.len() { best = Some(c); }
                } else {
                    best = Some(c);
                }
            }
        }
        best.cloned().unwrap_or_else(|| self.example_roma())
    }
}

fn expand_yure(base: &str, rules: &RomajiRules) -> Vec<String> {
    // Generate variants for common yure inside a word.
    let mut out: HashSet<String> = HashSet::new();
    out.insert(base.to_string());

    // Local replacement maps
    let pairs: [(&str, &str); 6] = [
        ("shi","si"), ("chi","ti"), ("tsu","tu"), ("ji","zi"), ("fu","hu"), ("sha","sya")
    ];
    let tri: [(&str, &str); 5] = [
        ("shu","syu"),
        ("sho","syo"),
        ("cha","tya"),
        ("chu","tyu"),
        ("cho","tyo"),
    ];

    for (a,b) in pairs { apply_bi(&mut out, a,b); apply_bi(&mut out, b,a); }
    for (a,b) in tri { apply_bi(&mut out, a,b); apply_bi(&mut out, b,a); }

    // "n'","nn","n" variants
    if let Some(sp) = &rules.special { 
        let mut extra: HashSet<String> = HashSet::new();
        for s in out.iter() {
            // Replace occurrences of n' with nn and n, and also allow final trailing n
            let mut forms = vec![s.clone()];
            if s.contains("n'") { 
                forms.push(s.replace("n'","nn"));
                forms.push(s.replace("n'","n"));
            }
            // heuristic: also allow replace "nn" -> "n'" and -> "n"
            if s.contains("nn") {
                forms.push(s.replace("nn","n'"));
                forms.push(s.replace("nn","n"));
            }
            for f in forms { extra.insert(f); }
        }
        out.extend(extra.into_iter());
    }

    out.into_iter().collect()
}

fn apply_bi(set: &mut HashSet<String>, a: &str, b: &str) {
    let mut add: HashSet<String> = HashSet::new();
    for s in set.iter() {
        if s.contains(a) { add.insert(s.replace(a,b)); }
    }
    set.extend(add.into_iter());
}
