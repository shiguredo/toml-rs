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

- [CHANGE] `no_std` に対応する
  - `alloc` クレートが必要
  - `HashMap` / `HashSet` を `BTreeMap` / `BTreeSet` に置き換える
  - `std::error::Error` を `core::error::Error` に置き換える
  - `PathSegment` に `PartialOrd`, `Ord` を追加する
  - @voluntas

### misc

- clippy `collapsible_match` 警告を解消する
  - @voluntas

## 2026.1.0

**リリース日**: 2026-02-26
