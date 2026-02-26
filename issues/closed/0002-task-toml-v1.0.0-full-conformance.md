# 0002: TOML v1.0.0 完全準拠対応

## タスク

`refs/v1.0.0.md` 準拠を再確認し、`src/` の実装を TOML v1.0.0 と整合させる。

## 実施内容

- `toml-test` (v1.0) の valid / invalid / encoder を用いて差分を抽出。
- 失敗 5 件を解消:
  - `valid/datetime/datetime`
  - `invalid/inline-table/duplicate-key-03`
  - `invalid/inline-table/overwrite-08`
  - `invalid/table/append-with-dotted-keys-01`
  - `invalid/table/append-with-dotted-keys-02`
- `src/parser.rs` を修正:
  - `[header]` で定義済みテーブルへの dotted key 拡張を禁止。
  - インラインテーブルで `key = {...}` 後の `key.sub = ...` を禁止。
- `src/datetime.rs` を修正:
  - `t` / `z` を受理。
  - `±HH:MM` の `MM` 範囲検証を追加。
- 回帰テスト追加:
  - `tests/test_parser.rs`
  - `tests/test_datetime.rs`

## 確認結果

- `make fmt`: 成功
- `make clippy`: 成功
- `make check`: 成功
- `make test`: 成功
- `toml-test` (TOML 1.0):
  - valid: 205 passed / 0 failed
  - encoder: 205 passed / 0 failed
  - invalid: 473 passed / 0 failed

