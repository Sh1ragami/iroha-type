use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use typewell_tui::engine::game::{Game, GameConfig, WordEntry};

fn key(c: char) -> KeyEvent { KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE) }

#[test]
fn simple_game_flow() {
    let words = vec![
        WordEntry{ jp: "あい".into(), romas: vec!["ai".into()] },
        WordEntry{ jp: "しお".into(), romas: vec!["shio".into()] },
    ];
    let mut g = Game::new(GameConfig{ time_limit_sec: 10.0, max_words: 10, loss_ms_per_miss: 0 }, words, std::path::Path::new("data/rules/romaji.yaml")).unwrap();
    g.start();
    for ch in "ai".chars() { g.handle_key(key(ch)).unwrap(); }
    for ch in "sio".chars() { g.handle_key(key(ch)).unwrap(); }
    assert!(g.finish_record().splits.len() >= 1);
}

