```
 _ ____            _          _   _           
(_)_  /  ___ _ __ | |__   ___| |_(_)_ __   __ 
 | |/ / / _ \ '_ \| '_ \ / _ \ __| | '_ \ / _|
 | / /_|  __/ |_) | | | |  __/ |_| | | | | (_ 
 |/____\___| .__/|_| |_|\___|\__|_|_| |_|\__|
            |_|  I r o h a T y p e
```

IrohaType — Japanese Typing TUI
================================

タイプウェル風の日本語タイピングアプリ。ターミナルで起動するTUIツールです。

[![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-b7410e.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)
![Edition](https://img.shields.io/badge/Rust_edition-2021-informational)
[![TUI](https://img.shields.io/badge/TUI-ratatui%200.27-00b4d8)](https://github.com/ratatui-org/ratatui)
[![crates.io](https://img.shields.io/crates/v/irohatype.svg)](https://crates.io/crates/irohatype)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![GitHub](https://img.shields.io/badge/github-Sh1ragami%2Firoha--type-181717?logo=github)](https://github.com/Sh1ragami/iroha-type)
[![Stars](https://img.shields.io/github/stars/Sh1ragami/iroha-type?style=social)](https://github.com/Sh1ragami/iroha-type)

デモ
----

https://github.com/user-attachments/assets/91c76305-8108-4afe-9b93-fc2d987c2297

インストール / 実行
------------------
前提: Rust (stable) がインストールされている必要があります。

1) crates.io からインストール
```
cargo install irohatype
irohatype
```

または、リポジトリから実行
```
git clone https://github.com/Sh1ragami/iroha-type
cd iroha-type
cargo run --release
```

はじめ方
--------
1) 画面が出たら `G` キーで開始
2) 終了後はポップアップに記録（秒数・順位）が表示
3) `Enter` でランキングへ / `ESC` でホームへ

遊び方
-------
- 上段の日本語を左から右へ。現在語は反転＋下線で位置が出ます
- 下段のローマ字をそのまま入力（`shi/si`, `cho/tyo` など表記ゆれOK）
- 誤タイプ時は、押した文字は出さず「正解の次文字」を赤で表示

操作方法
--------
- ホーム: `G` Start / `R` Ranking / `S` Settings / `Q` Quit
- プレイ: 文字キーで入力 / `ESC` 中断（中断時は記録保存しません）
- ポップアップ（終了後）: `Enter` ランキングへ / `ESC` ホームへ

フォルダ構成 / データ
------------------
- 設定: `data/config.json`
- 記録: `data/scores.json`
- 辞書: `data/words/basic_common.json`

ライセンス
--------
MIT
