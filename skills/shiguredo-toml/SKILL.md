---
name: shiguredo-toml
description: |
    Rust クレート shiguredo_toml を利用または変更するためのリポジトリ固有ガイド。
    TOML v1.0.0 / v1.1.0 のパース、シリアライズ、no_std、Document による非破壊編集、TextSpan とコメント位置、toml-test 準拠確認を扱う。
    shiguredo_toml、toml-rs、TOML の構文対応、値パス、位置情報、コメント保持、パーサやシリアライザの修正が話題になったときに使用する。
license: Apache-2.0
compatibility: Rust 1.93 以降と Cargo が必要。
metadata:
    github-repo: https://github.com/shiguredo/toml-rs
    github-path: skills/shiguredo-toml
---
# shiguredo_toml

`shiguredo_toml` の実際の API とリポジトリ構造に基づいて作業する。
一般的な TOML ライブラリの知識から API や挙動を推測しない。

## ライブラリの前提

- ルートクレートは `#![no_std]` であり、ヒープ割り当てに `alloc` を使用する
- ルートクレートの通常依存は 0 であり、`src/` の変更では `std` や外部クレートを持ち込まない
- MSRV は Rust 1.93、エディションは Rust 2024 である
- TOML v1.0.0 と v1.1.0 を扱う
- `Table` は `BTreeMap<String, Value>` なので、キー順はソートされる
- パーサとシリアライザの最大ネスト深度はどちらも 128 であり、片方を変える場合はもう片方との整合性も確認する

## API の選択

目的に応じて次の API を使う。

- TOML v1.0.0 として構造だけを読む場合は `from_str` を使う
- バージョンを明示して読む場合は `from_str_with_version` と `TomlVersion` を使う
- 構造から TOML を生成する場合は `to_string` または `to_string_pretty` を使う
- 単一値の TOML 表現が必要な場合は `to_inline_string` を使う
- 元のコメントや空白を維持しながら値を編集する場合は `Document` を使う

`from_str`、`Document::parse`、`Datetime::from_str` は TOML v1.0.0 として解析する。
現在の `Document` にはバージョン指定 API がないため、TOML v1.1.0 固有構文を含む文書の非破壊編集には使えない。

## パースとシリアライズ

`from_str` と `from_str_with_version` は、TOML 文書のルートを `Table` として返す。

```rust
use shiguredo_toml::{Error, TomlVersion, Value, from_str_with_version, to_string};

fn normalize(input: &str) -> Result<String, Error> {
    let table = from_str_with_version(input, TomlVersion::V1_1)?;
    to_string(&Value::Table(table))
}
```

`to_string` と `to_string_pretty` のトップレベル値は `Value::Table` でなければならない。
スカラー値や配列を直接渡すと `Error::Serialize` になる。

`Table` に変換した時点で、元のキー順、コメント、空白、引用方法は保持されない。
それらを保持する必要がある場合は、パース結果を再シリアライズせず `Document` を使う。

## 非破壊編集

`Document` は元テキスト、ルートテーブル、値範囲、コメント範囲、セクション範囲をまとめて保持する。
既存値の置換では値の範囲だけを書き換え、新規挿入では対象セクションにキー値行を追加した後、文書全体を再解析する。

```rust
use shiguredo_toml::{Document, Error, Value};

fn update_port(input: &str) -> Result<String, Error> {
    let mut document = Document::parse(input)?;
    document.set_path("servers[1].port", Value::Integer(9090))?;
    Ok(document.as_str().into())
}
```

文字列パスは `server.port` のようなキーと `servers[1]` のような配列インデックスを扱う。
文字列パスには TOML の引用キー構文やエスケープ構文がない。
ドットや角括弧を含むキーは、`PathSegment::Key` を組み立てて `get`、`set`、`span` に渡す。

既存の配列要素は、その値範囲が記録されていれば置換できる。
存在しない配列要素の挿入と、欠損部分を含む配列の自動作成はできない。
欠損している親がキーだけで構成される場合は、必要なセクションを自動作成できる。

編集が成功するたびに位置インデックスは再構築される。
編集前に取得した `TextSpan` は再利用せず、編集後の `Document` から取り直す。

## 位置情報とコメント

`TextSpan` は `start` を含み `end` を含まない UTF-8 バイト範囲である。
文字数や文字インデックスとして扱わず、同じ時点の `Document::as_str()` に対するスライスに使う。

```rust
use shiguredo_toml::{Document, Error, parse_value_path};

fn value_text<'a>(document: &'a Document, path: &str) -> Result<Option<&'a str>, Error> {
    let path = parse_value_path(path)?;
    Ok(document
        .span(&path)
        .map(|span| &document.as_str()[span.start..span.end]))
}
```

`CommentIndex::iter` は行末コメントと独立したコメントの両方を返す。
`CommentSpan::target` が `Some` になるのは、値に紐づいた行末コメントである。
`trailing_comment_span` と `trailing_comment_span_path` は、値に紐づいた行末コメントだけを返す。

## エラーの扱い

`Error` には `Parse`、`Serialize`、`Validate` がある。
`Error::position` が返す位置は、入力先頭からのバイト位置である。
解析エラーの表示には、元の入力を `get_line_and_column` と `get_line` に渡す。
行番号と列番号は 1 始まりである。

## ソースコードの対応関係

- `src/lib.rs`：公開 API と `TomlVersion`
- `src/parser.rs`：文法、TOML バージョン差、テーブル定義状態、値とコメントとセクションの範囲収集
- `src/serializer.rs`：通常出力、整形済み出力、インライン値、キーと文字列のエスケープ
- `src/edit.rs`：`Document`、値の置換、新規キーとセクションの挿入、編集後の再解析
- `src/span.rs`：`TextSpan`、値パス、値とコメントとセクションの各インデックス
- `src/datetime.rs`：4 種類の TOML 日時、検証、解析、表示
- `src/value.rs`：`Value`、`Table`、`Array` と型別アクセサ
- `src/error.rs`：エラー種別、バイト位置、行番号と列番号の算出
- `refs/v1.0.0.md` と `refs/v1.1.0.md`：実装判断の根拠にするローカル仕様書
- `tools/toml-test-adapter/`：公式 `toml-test` と公開 API を接続するアダプタ
- `pbt/`：`proptest` によるラウンドトリップと不変条件の検証
- `fuzz/`：パーサ、シリアライザ、編集、値パスのパニック安全性検証

## 実装時に維持する不変条件

TOML の構文や意味を変える前に、対象バージョンの `refs/` を確認する。
TOML v1.1.0 固有構文を追加するときは、`TomlVersion::V1_1` で受理し、`TomlVersion::V1_0` で拒否する境界テストを追加する。

テーブルの重複定義、暗黙テーブル、ドット付きキー、インラインテーブルの判定には `TableState` が関与する。
配列テーブルの判定には、専用のパス集合と現在要素インデックスも関与する。
ネストした `BTreeMap` への挿入だけで意味規則を代用しない。

配列テーブルの値パスには、現在の要素を示す `PathSegment::Index` が入る。
パーサを変える場合は、値の結果だけでなく、値範囲、行末コメントの対象、セクション範囲も確認する。

`Document` の編集は、生成した次のテキストを再解析できた場合だけ内部状態を更新する。
失敗時に元の `source`、`table`、各インデックスを部分更新しない。

公開日時構造体はフィールドを直接構築でき、シリアライザは各フィールドの `validate` を自動では呼ばない。
プログラムから日時を構築する場合は `Date::validate`、`Time::validate`、`Offset::validate` を呼び、日時バリアントの組み合わせも検証する。

## テストの選択

変更箇所に対応するテストを先に実行する。

- パーサ：`cargo test --test test_parser --test snapshot_parser`
- シリアライザ：`cargo test --test test_serializer --test snapshot_serializer`
- 非破壊編集と位置情報：`cargo test --test test_edit --test snapshot_edit`
- 日時：`cargo test --test test_datetime`
- 値とエラー：`cargo test --test test_value --test test_error`
- プロパティベーステスト：`cargo test -p pbt`

パーサ、シリアライザ、日時の意味を変えた場合は、公式テストスイートを両バージョンで実行する。

```bash
make toml-test-v1_0
make toml-test-v1_1
```

公式テストの初回実行には Go とネットワーク接続が必要であり、テストスイートは `.cache/toml-test` に取得される。
スナップショット差分は期待する仕様変更と一致するかを確認し、失敗を解消する目的だけで一括更新しない。

パニック安全性や入力空間に関わる変更では、対応する `fuzz/fuzz_targets/` を拡張し、対象ターゲットを実行する。

```bash
cargo +nightly fuzz run fuzz_parse -- -max_total_time=30 -max_len=4096
```

## 完了前の確認

Rust コードを変更した場合は、リポジトリの `AGENTS.md` と `shiguredo-rust` スキルにも従う。
最終確認は CI と同じ順序で実行する。

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

公開 API や利用上の制約を変えた場合は、`README.md`、`rustdoc`、変更履歴の更新要否も確認する。
