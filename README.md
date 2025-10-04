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

タイプウェル風のTUIタイピングアプリです。

[![Made with Rust](https://img.shields.io/badge/Made%20with-Rust-b7410e.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)
![Edition](https://img.shields.io/badge/Rust_edition-2021-informational)
[![TUI](https://img.shields.io/badge/TUI-ratatui%200.27-00b4d8)](https://github.com/ratatui-org/ratatui)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)

デモ
----

https://github.com/user-attachments/assets/91c76305-8108-4afe-9b93-fc2d987c2297

できること
--------
- 常用語をテンポよく練習（上: 日本語／下: ローマ字）
- 誤タイプ時は「次に打つべき1文字」を赤でヒント
- 既定は「400打」で打ち切り。記録は自動で保存・ランキング（Top 100）に反映

インストール / 実行
------------------
前提: Rust (stable) が入っていればOK

```
git clone <このリポジトリ>
cd <repo>
cargo run --release
```

はじめかた
--------
1) ターミナルでこのフォルダを開く
2) `cargo run` を実行（上と同じ）
3) 画面が出たら G キーで開始


トラブルシュート
--------------
- 画面が狭い: 設定でステージを小さく（←/→, ↑/↓）
- 文字が□で出る: 端末フォントを日本語対応に／UTF-8 を有効に
- ローマ字が通らない: `shi/si`, `cha/tya`, `cho/tyo` など複数表記に対応。別表記を試してください

フォルダ構成 / データ保存
---------------------
- 設定: `data/config.json`（画面サイズ、打鍵数など）
- 記録: `data/scores.json`（ランキング）
- 辞書: `data/words/basic_common.json`

ライセンス
--------
MIT
