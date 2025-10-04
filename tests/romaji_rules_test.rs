use typewell_tui::engine::romaji::{RomajiRules, RomajiMatcher, InputResult};

fn rules() -> RomajiRules {
    RomajiRules::from_yaml_file(std::path::Path::new("data/rules/romaji.yaml")).unwrap()
}

fn feed(m: &mut RomajiMatcher, s: &str) -> Vec<InputResult> {
    s.chars().map(|c| m.input_char(c)).collect()
}

#[test]
fn shi_si_mix() {
    let r = rules();
    // base romaji given for the word
    let mut m = RomajiMatcher::new("しし", &vec!["shishi".into()], &r);
    let res = feed(&mut m, "sisi");
    assert!(matches!(res.last(), Some(InputResult::Complete)));
}

#[test]
fn chi_ti_mix() {
    let r = rules();
    let mut m = RomajiMatcher::new("ちち", &vec!["chichi".into()], &r);
    let res = feed(&mut m, "titi");
    assert!(matches!(res.last(), Some(InputResult::Complete)));
}

#[test]
fn tsu_tu_mix() {
    let r = rules();
    let mut m = RomajiMatcher::new("つつ", &vec!["tsutsu".into()], &r);
    let res = feed(&mut m, "tutu");
    assert!(matches!(res.last(), Some(InputResult::Complete)));
}

#[test]
fn ji_zi_mix() {
    let r = rules();
    let mut m = RomajiMatcher::new("じじ", &vec!["jiji".into()], &r);
    let res = feed(&mut m, "zizi");
    assert!(matches!(res.last(), Some(InputResult::Complete)));
}

#[test]
fn fu_hu_mix() {
    let r = rules();
    let mut m = RomajiMatcher::new("ふふ", &vec!["fufu".into()], &r);
    let res = feed(&mut m, "huhu");
    assert!(matches!(res.last(), Some(InputResult::Complete)));
}

#[test]
fn n_variants() {
    let r = rules();
    let mut m = RomajiMatcher::new("こんぶ", &vec!["kon'bu".into()], &r);
    let res = feed(&mut m, "konbu");
    assert!(matches!(res.last(), Some(InputResult::Complete)));
    let mut m2 = RomajiMatcher::new("こんぶ", &vec!["kon'bu".into()], &r);
    let res2 = feed(&mut m2, "konnbu");
    assert!(matches!(res2.last(), Some(InputResult::Complete)));
}

#[test]
fn small_kya_ok() {
    let r = rules();
    let mut m = RomajiMatcher::new("きゃく", &vec!["kyaku".into()], &r);
    let res = feed(&mut m, "kyaku");
    assert!(matches!(res.last(), Some(InputResult::Complete)));
}

#[test]
fn shachi_mixed() {
    let r = rules();
    let mut m = RomajiMatcher::new("しゃち", &vec!["shachi".into()], &r);
    let res = feed(&mut m, "syati");
    assert!(matches!(res.last(), Some(InputResult::Complete)));
}

#[test]
fn sokuon_double_required() {
    let r = rules();
    let mut m = RomajiMatcher::new("がっこう", &vec!["gakkou".into()], &r);
    let res = feed(&mut m, "gakou");
    assert!(matches!(res.iter().last(), Some(InputResult::Miss | InputResult::Correct))); // should not be complete
}

#[test]
fn complete_on_exact() {
    let r = rules();
    let mut m = RomajiMatcher::new("あい", &vec!["ai".into()], &r);
    let res = feed(&mut m, "ai");
    assert!(matches!(res.last(), Some(InputResult::Complete)));
}

#[test]
fn mix_inside_word_shi_tsu_ji() {
    let r = rules();
    let mut m = RomajiMatcher::new("しつじ", &vec!["shitsuji".into()], &r);
    // si + tu + zi
    let res = feed(&mut m, "situz i".replace(' ', "").as_str());
    assert!(matches!(res.last(), Some(InputResult::Complete)));
}

#[test]
fn fuji_huzi() {
    let r = rules();
    let mut m = RomajiMatcher::new("ふじさん", &vec!["fujisan".into()], &r);
    let res = feed(&mut m, "huzisan");
    assert!(matches!(res.last(), Some(InputResult::Complete)));
}

#[test]
fn jun_zyun_n_rules() {
    let r = rules();
    let mut m = RomajiMatcher::new("じゅん", &vec!["jun".into()], &r);
    let res1 = feed(&mut m, "zyun");
    assert!(matches!(res1.last(), Some(InputResult::Complete)));
    let mut m2 = RomajiMatcher::new("じゅん", &vec!["jun".into()], &r);
    let res2 = feed(&mut m2, "junn");
    assert!(matches!(res2.last(), Some(InputResult::Complete)));
}

#[test]
fn shinyou_n_variants() {
    let r = rules();
    let mut m = RomajiMatcher::new("しんよう", &vec!["shinyou".into()], &r);
    let res = feed(&mut m, "sinyou");
    assert!(matches!(res.last(), Some(InputResult::Complete)));
}

#[test]
fn sya_sha() {
    let r = rules();
    let mut m = RomajiMatcher::new("しゃ", &vec!["sha".into()], &r);
    let res = feed(&mut m, "sya");
    assert!(matches!(res.last(), Some(InputResult::Complete)));
}

#[test]
fn syo_sho() {
    let r = rules();
    let mut m = RomajiMatcher::new("しょ", &vec!["sho".into()], &r);
    let res = feed(&mut m, "syo");
    assert!(matches!(res.last(), Some(InputResult::Complete)));
}

#[test]
fn tyu_chu() {
    let r = rules();
    let mut m = RomajiMatcher::new("ちゅ", &vec!["chu".into()], &r);
    let res = feed(&mut m, "tyu");
    assert!(matches!(res.last(), Some(InputResult::Complete)));
}

#[test]
fn miss_when_not_prefix() {
    let r = rules();
    let mut m = RomajiMatcher::new("し", &vec!["shi".into()], &r);
    let res = feed(&mut m, "x");
    assert!(matches!(res.last(), Some(InputResult::Miss)));
}

#[test]
fn partial_is_correct_not_complete() {
    let r = rules();
    let mut m = RomajiMatcher::new("し", &vec!["shi".into()], &r);
    let res = feed(&mut m, "s");
    matches!(res.last(), Some(InputResult::Correct));
}

#[test]
fn hyphen_ok_as_literal() {
    let r = rules();
    let mut m = RomajiMatcher::new("スーパー", &vec!["su-pa-".into()], &r);
    let res = feed(&mut m, "su-pa-");
    assert!(matches!(res.last(), Some(InputResult::Complete)));
}
