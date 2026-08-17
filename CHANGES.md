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
- [CHANGE] PBT の依存を proptest から noprop に切り替える
  - @voluntas
- [FIX] 配列テーブルを再オープンした後にサブテーブルを定義するとエラーになるのを修正する
  - @voluntas
- [FIX] 無効な Datetime をシリアライズするとエラーになるようにする
  - `Datetime::validate` を追加し、シリアライズ時に検証する
  - @voluntas

### misc

- PBT を noprop の機能を使い切る書式に改善する
  - `sample_with_boundaries` で境界値を一定の割合で混入する
  - coverage gate を追加し、検証対象が空振りしないことを保証する
  - 生成ヘルパーに `#[track_caller]` を付与して失敗トレースの位置を呼び出し元に向ける
  - 失敗時の assert メッセージに生成値と入力文字列を含める
  - @voluntas
- `unwrap()` を `expect()` に置き換える
  - @voluntas
- prek に tombi フックを追加し、cargo test を pre-push に移動する
  - @voluntas

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
