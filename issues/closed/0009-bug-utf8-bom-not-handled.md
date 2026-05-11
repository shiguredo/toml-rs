# UTF-8 BOM が valid 入力として受け入れられない

Created: 2026-05-08
Completed: 2026-05-08
Model: Opus 4.7

## 概要

入力先頭の UTF-8 BOM (`\xEF\xBB\xBF`、`U+FEFF`) を読み飛ばさず、パースエラーになる。
toml-test (TOML 1.0 spec) の `valid/utf8-bom-01` および `valid/utf8-bom-02` が失敗している。

## 根拠

- toml-test (BurntSushi/toml v1.6.0 連携) で `valid/utf8-bom-01`, `valid/utf8-bom-02` が valid 扱いになっており、BOM 付き入力はパース成功（出力は BOM を含まない `a=1` 相当）が要求される。
- CI 失敗実例: GitHub Actions run 25534603524 / job 74947747744
  - `valid tests: 207 passed, 2 failed` の 2 件が上記 BOM 関連テスト。
- TOML 1.0.0 仕様自体は BOM の扱いを明記していないが、toml-test 互換実装としては BOM を許容する必要がある。

## 再現手順

```bash
printf '\xEF\xBB\xBFa=1\n' \
  | cargo run --manifest-path tools/toml-test-adapter/Cargo.toml --bin toml-test-decoder
```

現状: 非 0 終了（`Exit 1`、パースエラー）。
期待: 0 終了 + `a` がキー、値が整数 `1` の JSON が出力される。

または `make toml-test-v1_0` を実行すると `valid/utf8-bom-01`, `valid/utf8-bom-02` で FAIL となる。

## 期待動作

- 入力先頭が UTF-8 BOM (`U+FEFF`) の場合、その 3 バイトを読み飛ばして通常どおりパースする。
- BOM は先頭 1 個のみを許容し、先頭以外（ファイル中段、行頭以外、複数連続）に出現した場合は従来どおりパースエラーとする。

## 影響範囲

- `src/parser.rs` の `parse_with_spans`（パーサ初期化部）
- BOM 受け入れと BOM 非先頭時のエラーに関するテスト追加
- `CHANGES.md` に `[FIX]` エントリ追加

## 解決方法

`src/parser.rs::parse_with_spans` で `Parser` を初期化する際、`rest` のみ
`str::strip_prefix('\u{FEFF}')` で先頭 BOM を剥がすようにした。`input` は
元のまま保持しているため、`position()` の値（= `input.len() - rest.len()`）
や `SpanIndex` / `CommentIndex` / `SectionIndex` のオフセットは元入力基準で
一貫し、エラー位置や span の整合性も保たれる。

BOM の許容は先頭の 1 個のみ。2 個目以降や中間に出現した BOM は従来どおり
パースエラーになる。

- 修正: `src/parser.rs` の `parse_with_spans` 入口に BOM スキップを追加
- テスト: `tests/test_parser.rs` の `mod bom` に valid/invalid 6 ケースを追加
  - 先頭 BOM + 改行 + コメント (valid/utf8-bom-01 相当)
  - 先頭 BOM + キー=値 + 行末コメント (valid/utf8-bom-02 相当)
  - BOM のみで内容空
  - BOM 連続はエラー
  - 中間 BOM はエラー
  - 値中の BOM はエラー
- toml-test (TOML 1.0): valid 209/209, encoder 209/209, invalid 493/493
- toml-test (TOML 1.1): valid 218/218, encoder 218/218, invalid 486/486
