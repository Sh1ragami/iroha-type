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
    pub fn from_yaml_str(s: &str) -> Result<Self> {
        let mut r: RomajiRules = serde_yaml::from_str(s)?;
        if r.special.is_none() { r.special = Some(SpecialRules{ n_patterns: vec!["n'".into(), "nn".into(), "n".into()] }); }
        Ok(r)
    }
}

pub enum InputResult { Correct, Miss, Complete, Noop }

pub struct RomajiMatcher {
    _jp: String,
    candidates: Vec<String>,
    pub typed: String,
    pub miss_count: u32,
    current_cand: Option<usize>,
}

impl RomajiMatcher {
    pub fn new(jp: &str, base_romas: &[String], rules: &RomajiRules) -> Self {
        let mut set: HashSet<String> = HashSet::new();
        for b in base_romas {
            for v in expand_yure(b, rules) { set.insert(v); }
        }
        let mut candidates: Vec<String> = set.into_iter().collect();
        candidates.sort_by_key(|s| s.len());
        Self{ _jp: jp.into(), candidates, typed: String::new(), miss_count: 0, current_cand: None }
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
        if any_exact {
            self.typed = next;
            self.update_display_candidate();
            return InputResult::Complete;
        }
        if any_prefix {
            self.typed = next;
            self.update_display_candidate();
            return InputResult::Correct;
        }
        self.miss_count += 1; InputResult::Miss
    }

    pub fn example_roma(&self) -> String {
        self.candidates.first().cloned().unwrap_or_default()
    }

    pub fn display_candidate(&self) -> String {
        // Sticky candidate: if current_cand still matches typed prefix, keep it.
        let pref = &self.typed;
        if pref.is_empty() { return self.example_roma(); }
        if let Some(i) = self.current_cand {
            if let Some(c) = self.candidates.get(i) { if c.starts_with(pref) { return c.clone(); } }
        }
        // Fallback: choose shortest matching candidate
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

    pub fn display_with_default(&self, default: &str) -> String {
        // Keep default until switch is confirmed and necessary
        let pref = &self.typed;
        if pref.is_empty() { return default.to_string(); }
        if default.starts_with(pref) { return default.to_string(); }
        if !is_confirmed_switch(pref) { return default.to_string(); }
        self.display_candidate()
    }

    fn update_display_candidate(&mut self) {
        // Set current_cand to a stable choice among candidates matching typed prefix.
        let pref = &self.typed;
        if pref.is_empty() { self.current_cand = None; return; }
        if let Some(i) = self.current_cand {
            if let Some(c) = self.candidates.get(i) { if c.starts_with(pref) { return; } }
        }
        // Choose shortest candidate that matches
        let mut best_idx: Option<usize> = None;
        for (i, c) in self.candidates.iter().enumerate() {
            if c.starts_with(pref) {
                if let Some(bi) = best_idx {
                    if c.len() < self.candidates[bi].len() { best_idx = Some(i); }
                } else {
                    best_idx = Some(i);
                }
            }
        }
        self.current_cand = best_idx;
    }
}

fn is_confirmed_switch(pref: &str) -> bool {
    // Heuristic: require at least 2 chars to flip away from default
    // so that a single 't' or 'j' doesn't immediately rewrite to tyu/jyu, etc.
    pref.len() >= 2
}

fn expand_yure(base: &str, _rules: &RomajiRules) -> Vec<String> {
    // Generate variants for common yure inside a word.
    let mut out: HashSet<String> = HashSet::new();
    out.insert(base.to_string());

    // Local replacement maps（一般的なものに限定）
    // 対象: し/ち/つ/じ と 拗音の s/ty/j 系のみ
    let pairs: [(&str, &str); 5] = [
        ("shi","si"), ("chi","ti"), ("tsu","tu"), ("ji","zi"), ("sha","sya"),
    ];
    let tri: [(&str, &str); 10] = [
        ("shu","syu"),
        ("sho","syo"),
        ("cha","tya"),
        ("chu","tyu"),
        ("cho","tyo"),
        ("ja","jya"), ("ju","jyu"), ("jo","jyo"),
        // z 系（zyu等）
        ("jyu","zyu"),
        ("jyo","zyo"),
    ];

    for (a,b) in pairs { apply_bi(&mut out, a,b); apply_bi(&mut out, b,a); }
    for (a,b) in tri { apply_bi(&mut out, a,b); apply_bi(&mut out, b,a); }

    // ふ: fu -> hu は、前が f でない場合のみ生成（ffu→fhuのような不自然形を避ける）
    apply_fu_to_hu_safe(&mut out);
    // っ + ふ: ffu と hhu は相互に許可
    apply_bi(&mut out, "ffu", "hhu"); apply_bi(&mut out, "hhu", "ffu");

    // 'ん'の扱い: 一般時は n/nn 両方許可、母音・yの直前は nn のみ
    // 辞書の基本表記に従い、以下の拡張のみ行う:
    //  - 子音前の単独 n を nn に置換したバリアントを追加（逆方向はしない）
    apply_n_doubling_before_cons(&mut out);

    out.into_iter().collect()
}

fn apply_bi(set: &mut HashSet<String>, a: &str, b: &str) {
    let mut add: HashSet<String> = HashSet::new();
    for s in set.iter() {
        if s.contains(a) { add.insert(s.replace(a,b)); }
    }
    set.extend(add.into_iter());
}

fn apply_fu_to_hu_safe(set: &mut HashSet<String>) {
    let mut add: HashSet<String> = HashSet::new();
    for s in set.iter() {
        let bytes = s.as_bytes();
        let mut i = 0;
        while i + 1 < bytes.len() {
            if bytes[i] == b'f' && bytes[i+1] == b'u' {
                // if preceded by 'f', skip (this is likely 'ffu')
                if i == 0 || bytes[i-1] != b'f' {
                    let mut ns = String::with_capacity(s.len());
                    ns.push_str(&s[..i]);
                    ns.push('h');
                    ns.push('u');
                    ns.push_str(&s[i+2..]);
                    add.insert(ns);
                }
                i += 2; continue;
            }
            i += 1;
        }
    }
    set.extend(add.into_iter());
}

fn apply_n_doubling_before_cons(set: &mut HashSet<String>) {
    let mut add: HashSet<String> = HashSet::new();
    for s in set.iter() {
        let bytes = s.as_bytes();
        for i in 0..bytes.len().saturating_sub(1) {
            if bytes[i] == b'n' {
                let next = bytes[i+1] as char;
                if next.is_ascii_alphabetic() {
                    let is_vowel = matches!(next, 'a'|'i'|'u'|'e'|'o');
                    if !is_vowel && next != 'y' && next != 'n' {
                        // insert an extra 'n' at position i
                        let mut ns = String::with_capacity(s.len()+1);
                        ns.push_str(&s[..i]);
                        ns.push('n');
                        ns.push_str(&s[i..]);
                        add.insert(ns);
                    }
                }
            }
        }
    }
    set.extend(add.into_iter());
}
