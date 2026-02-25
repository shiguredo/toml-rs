# 0005: toml_edit 相当の編集基盤を追加する

## タスク

`toml_edit` 相当の非破壊編集基盤を追加し、各値が元テキスト上の位置情報を保持する設計にする。

## 実施内容

- 値位置情報の型を追加:
  - `TextSpan`
  - `PathSegment`
  - `ValuePath`
  - `SpanIndex`
  - `parse_value_path()`
- パーサを拡張:
  - `parse_with_spans()` を追加
  - 値パスごとの `TextSpan` を収集
  - 配列・インラインテーブル内のネスト値にもパスを付与
  - 配列テーブル (`[[...]]`) の現在要素インデックスを追跡
- 編集 API を追加:
  - `Document::parse()`
  - `Document::get()` / `get_path()`
  - `Document::span()`
  - `Document::set()` / `set_path()`
  - 置換後は再パースしてテーブルと位置情報を同期
- 追加 API:
  - `to_inline_string()`
  - `Document` の `FromStr` 実装
- テスト追加:
  - 単体: `tests/test_edit.rs`
  - PBT: `pbt/tests/prop_edit.rs`
- README に編集 API の利用例を追記。

## 確認結果

- `make fmt`: 成功
- `make clippy`: 成功
- `make check`: 成功
- `make test`: 成功
- `make toml-test-ci`: valid 205 / encoder 205 / invalid 473 すべて成功

