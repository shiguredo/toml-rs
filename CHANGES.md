# 変更履歴

- CHANGE
  - 下位互換のない変更
- UPDATE
  - 下位互換がある変更
- ADD
  - 下位互換がある追加
- FIX
  - バグ修正

## develop

- [CHANGE] 最小対応 Rust バージョン (MSRV) を 1.88 から 1.93 に引き上げる
  - @voluntas

### misc

## 2026.2.0

**リリース日**: 2026-05-11

- [CHANGE] `no_std` に対応する
  - `alloc` クレートが必要
  - `HashMap` / `HashSet` を `BTreeMap` / `BTreeSet` に置き換える
  - `std::error::Error` を `core::error::Error` に置き換える
  - `PathSegment` に `PartialOrd`, `Ord` を追加する
  - @voluntas
- [FIX] 入力先頭の UTF-8 BOM (U+FEFF) を読み飛ばす
  - toml-test の `valid/utf8-bom-01`, `valid/utf8-bom-02` に対応する
  - @voluntas

### misc

- clippy `collapsible_match` 警告を解消する
  - @voluntas

## 2026.1.0

**リリース日**: 2026-02-26
