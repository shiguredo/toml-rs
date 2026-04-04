# toml-rs

[![shiguredo_toml](https://img.shields.io/crates/v/shiguredo_toml.svg)](https://crates.io/crates/shiguredo_toml)
[![Documentation](https://docs.rs/shiguredo_toml/badge.svg)](https://docs.rs/shiguredo_toml)
[![GitHub Actions](https://github.com/shiguredo/toml-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/shiguredo/toml-rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

## About Shiguredo's open source software

We will not respond to PRs or issues that have not been discussed on Discord. Also, Discord is only available in Japanese.

Please read <https://github.com/shiguredo/oss> before use.

## 時雨堂のオープンソースソフトウェアについて

利用前に <https://github.com/shiguredo/oss> をお読みください。

## 概要

Rust で実装された依存 0 の TOML ライブラリです。TOML v1.0.0 と v1.1.0 の両方に対応しています。

## 特徴

- TOML v1.0.0 / v1.1.0 仕様に完全準拠
- TOML 公式テストスイート [toml-test](https://github.com/toml-lang/toml-test) 全件パス
- 依存ライブラリ 0
- パーサとシリアライザを提供
- 整形済み出力 (pretty print) 対応
- 値の位置情報 (バイト範囲) を保持
- コメントの位置情報を保持し、値との紐づけを提供
- 位置情報を活用した非破壊編集
- `no_std` 対応

## 使い方

### パース

`from_str` は TOML v1.0.0 としてパースします。

```rust
use shiguredo_toml::{Error, from_str};

fn main() -> Result<(), Error> {
    let table = from_str(r#"
[server]
host = "localhost"
port = 8080
enabled = true

[[servers]]
name = "alpha"
ip = "10.0.0.1"

[[servers]]
name = "beta"
ip = "10.0.0.2"
"#)?;

    assert_eq!(
        table.get("server")
            .and_then(|v| v.get("host"))
            .and_then(|v| v.as_str()),
        Some("localhost")
    );
    assert_eq!(
        table.get("server")
            .and_then(|v| v.get("port"))
            .and_then(|v| v.as_integer()),
        Some(8080)
    );
    assert_eq!(
        table.get("servers")
            .and_then(|v| v.as_array())
            .map(|servers| servers.len()),
        Some(2)
    );

    Ok(())
}
```

### バージョン指定でパース

`from_str_with_version` で TOML バージョンを明示できます。

```rust
use shiguredo_toml::{Error, TomlVersion, from_str_with_version};

fn main() -> Result<(), Error> {
    // TOML v1.0.0 として解析する
    let _table_v1_0 = from_str_with_version(r#"
host = "localhost"
port = 8080
"#, TomlVersion::V1_0)?;

    // TOML v1.1.0 として解析する
    // 複数行インラインテーブル・末尾カンマ・\e エスケープ等が使用可能
    let _table_v1_1 = from_str_with_version(r#"
contact = {
    name = "Alice",
    email = "alice@example.com",
}
"#, TomlVersion::V1_1)?;

    Ok(())
}
```

### シリアライズ

```rust
use shiguredo_toml::{Error, Table, Value, to_string, to_string_pretty};

fn main() -> Result<(), Error> {
    let mut table = Table::new();
    table.insert("name".into(), Value::String("example".into()));
    table.insert("version".into(), Value::Integer(1));

    let value = Value::Table(table);
    let s = to_string(&value)?;
    let s_pretty = to_string_pretty(&value)?;
    assert!(!s.is_empty());
    assert!(!s_pretty.is_empty());

    Ok(())
}
```

### 日時

```rust
use shiguredo_toml::{Error, from_str};

fn main() -> Result<(), Error> {
    let table = from_str(r#"
odt = 1979-05-27T07:32:00Z
ldt = 1979-05-27T07:32:00
ld = 1979-05-27
lt = 07:32:00
"#)?;

    assert_eq!(
        table.get("odt")
            .and_then(|v| v.as_datetime())
            .and_then(|dt| dt.date.as_ref())
            .map(|d| d.year),
        Some(1979)
    );

    Ok(())
}
```

### 非破壊編集

`Document` はパース時にすべての値のバイト範囲 (`TextSpan`) とコメントの位置情報を記録します。
値を更新しても、位置情報に基づいて該当箇所だけを置換し再解析するため、コメントやフォーマットが失われません。

```rust
use shiguredo_toml::{Document, Error, Value};

fn main() -> Result<(), Error> {
    let mut doc = Document::parse("port = 8080 # keep\n")?;

    // 値の位置情報を使って該当箇所だけを置換する
    doc.set_path("port", Value::Integer(9090))?;

    // コメントの位置情報も更新後のテキストに追従する
    let comment_span = doc.trailing_comment_span_path("port");

    assert_eq!(doc.as_str(), "port = 9090 # keep\n");
    assert!(comment_span.is_some());
    if let Some(span) = comment_span {
        assert_eq!(&doc.as_str()[span.start..span.end], "# keep");
    }

    Ok(())
}
```

### 位置情報とコメント情報の取得

パース結果から値やコメントのバイト範囲を直接参照できます。

```rust
use shiguredo_toml::{Document, Error};

fn main() -> Result<(), Error> {
    let doc = Document::parse(r#"
[server]
host = "localhost" # primary
port = 8080
"#)?;

    // 値の位置情報 (バイト範囲) を取得する
    let span = doc.trailing_comment_span_path("server.host");
    assert!(span.is_some());
    if let Some(span) = span {
        assert_eq!(&doc.as_str()[span.start..span.end], "# primary");
    }

    // すべてのコメントを列挙する
    for comment in doc.comments().iter() {
        let text = &doc.as_str()[comment.span.start..comment.span.end];
        println!("{text}");
    }

    Ok(())
}
```

主な API:

- `doc.get_path("servers[1].name")`: 文字列パスで値を取得
- `doc.span(path)`: 値の `TextSpan` (バイト範囲) を取得
- `doc.trailing_comment_span_path("port")`: 行末コメントの `TextSpan` を取得
- `doc.comments()`: 収集済みコメントの一覧を取得
- `doc.spans()`: すべての値の位置情報インデックスを取得


## 対応する TOML の型

- 文字列 (基本文字列、リテラル文字列、複数行文字列)
- 整数 (10 進数、16 進数、8 進数、2 進数)
- 浮動小数点数 (inf、nan 含む)
- ブール値
- 日時 (オフセット付き日時、ローカル日時、ローカル日付、ローカル時刻)
- 配列
- テーブル (標準テーブル、インラインテーブル、配列テーブル)

## 規格書

このライブラリが準拠している仕様です。

- TOML v1.0.0
  - <https://toml.io/en/v1.0.0>
- TOML v1.1.0
  - <https://toml.io/en/v1.1.0>

## 準拠テスト (toml-test)

TOML 公式テストスイート `toml-test` を 1 コマンドで実行できます。

```bash
# TOML v1.0.0 で実行 (valid / invalid / encoder を一括実行)
make toml-test-v1_0

# TOML v1.1.0 で実行
make toml-test-v1_1

# 日時系だけを素早く実行
make toml-test-time

# CI 向け実行
make toml-test-v1_0-ci
make toml-test-v1_1-ci
```

補足:

- 初回は `scripts/run_toml_test.sh` が `toml-test` を `.cache/toml-test` に clone します。
- `TOML_TEST_UPDATE=1 make toml-test-v1_0` / `TOML_TEST_UPDATE=1 make toml-test-v1_1` で `toml-test` の更新を取り込みます。
- `go` コマンドが必要です。

## ライセンス

Apache License 2.0

```text
Copyright 2026-2026, Shiguredo Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
