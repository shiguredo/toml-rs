# 0006 task track-comment-spans-for-edit

## 背景

`Document` は値の位置情報のみを保持しており、コメント位置情報を直接参照できない。
`edit` 時の検証強化のため、コメント位置情報の追跡が必要。

## 対応内容

- `CommentSpan` と `CommentIndex` を追加する
- パーサでコメントのバイト範囲を記録する
- 行末コメントは値パスに紐づける
- `Document` からコメント位置情報へアクセスできる API を追加する
- `UTF-8`、`array of tables`、再編集時のずれ検証を含むテストを追加する

## 完了条件

- `make fmt` / `make clippy` / `make check` / `make test` / `make toml-test-ci` が通る
- コメント位置情報を使う単体テストが追加されている

## 完了内容

- `src/span.rs` に `CommentSpan` / `CommentIndex` を追加した
- `src/parser.rs` でコメント範囲を収集し、行末コメントを値パスへ関連付けた
- `src/edit.rs` に `comments` / `trailing_comment_span` / `trailing_comment_span_path` を追加した
- `tests/test_edit.rs` にコメント追跡の厳密テストを追加した
- すべての必須コマンドが成功した
