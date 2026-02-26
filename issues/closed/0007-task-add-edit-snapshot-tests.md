# 0007-task-add-edit-snapshot-tests

## 背景

`Document` の `edit` 機能向けに、出力テキストだけでなく位置情報の挙動も含めて差分検知できる `snapshot` テストが不足している。

## 対応内容

- `tests/snapshot_edit.rs` を追加する
- 値置換後の TOML テキストの `snapshot` を追加する
- 値 span / コメント span を整形して `snapshot` する

## 完了条件

- `make fmt` / `make clippy` / `make check` / `make test` が成功する
- `tests/snapshot_edit.rs` の `snapshot` が生成されている

## 完了内容

- `tests/snapshot_edit.rs` を追加した
- 値置換結果の `snapshot` を追加した
- 値 span / コメント span の整形 `snapshot` を追加した
- `make fmt` / `make clippy` / `make check` / `make test` が成功した
