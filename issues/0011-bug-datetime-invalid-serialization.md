# 無効な Datetime からシリアライザが Ok のまま無効な TOML を生成する

- Created: 2026-08-06
- Completed: {YYYY-MM-DD}
- Branch: feature/fix-datetime-serialization-validation
- Polished: 2026-08-06

## 目的

公開 API 経由で構築された無効な `Datetime` / `Date` / `Time` に対して、シリアライザが `Ok` を返しながら再パース不能な TOML を生成したり、オフセット情報を黙って破棄したりする経路を塞ぐ。

## 現状

`Datetime` / `Date` / `Time` のフィールドはすべて `pub`（src/datetime.rs）で、無効な組み合わせを型システムで防げない。一方、シリアライズ経路では `validate()` が呼ばれないため、以下の問題が発生する:

1. `Datetime { date: None, time: None, offset: Some(...) }` は `Datetime` の `Display` 実装（src/datetime.rs）の `(None, None, _)` アームで空文字を出力し、`Serializer::write_inline_value`（src/serializer.rs）がそれをそのまま出力する。`to_string` / `to_string_pretty` はテーブルに包んだ場合に `key = ` のような再パース不能な TOML を、`to_inline_string` は空文字列を `Ok` で返す（値が消失する）
2. `Datetime { date: Some(...), time: None, offset: Some(...) }` や `Datetime { date: None, time: Some(...), offset: Some(...) }` は `Display` がオフセットを出力せず、`1979-05-27` のように情報が黙って失われる（データ損失）
3. 範囲外の値（例: `Date { month: 13, day: 40 }`）は `2024-13-40T01:02:03` のような再パース不能な TOML を `Ok` で生成する

パース経路では常に検証済みの値が生成されるため、いずれも利用者が `Datetime` 等を直接構築した場合にのみ発生する。なお、`Document::set` / `set_path`（src/edit.rs）は内部で `to_inline_string` を呼ぶため、この経路でも同じ問題（オフセット黙殺）が発生する。

## 設計方針

シリアライザ経路で `Datetime` の検証を行い、無効な値に対しては `Error::Serialize` を返す。

`Date::validate` / `Time::validate` / `Offset::validate` は既に `pub fn` として実装されている（src/datetime.rs）が、`Datetime` レベルでの検証は存在しない。`Datetime::validate` を新規実装し、各フィールドの `validate()` 呼び出しに加えて、TOML v1.0.0 仕様の 4 バリアント（Offset Date-Time / Local Date-Time / Local Date / Local Time。refs/v1.0.0.md の日時節）以外の組み合わせ（date と time の両方が None、または offset が Some で date か time の片方が None）を拒否する。

`Date::validate` は year を検証していない（`Date { year: 10000, ... }` は validate を通過し、`10000-01-01` は再パース不能になる）。RFC 3339 の date-fullyear = 4DIGIT（外部仕様。refs/v1.0.0.md は日時を RFC 3339 に委譲している）であるため、`Date::validate` に year ≤ 9999 のチェックを追加する。

`Display` 実装の `(None, None, _)` アームは変更しない。`fmt::Error` を返す Display は `to_string()` / `format!` で panic を誘発し、無効な `Datetime` は公開 API 経由で構築可能であるため panic は不適切である。「削除する」は match の網羅性（src/datetime.rs の `Display` 実装）で不可能である。シリアライザ経路の validate で無効値を先に弾けば `Display` に無効値が到達しないため、空文字出力はシリアライザ経路の無効 TOML の温床にならない。

## 完了条件

- 無効な `Datetime`（不正な `Date` / `Time` / `Offset`、または 4 バリアントに該当しない組み合わせ）を値に含む `Value::Table` を `to_string` / `to_string_pretty` に、単体の `Value::Datetime` を `to_inline_string` に渡すと `Err`（`Error::Serialize`）が返る
- シリアライザが `Ok` を返しながら再パース不能な TOML を生成する経路が存在しない（解決方法のテストで検証する）
- パース経路で生成される値（valid な Datetime）のシリアライズは従来どおり成功する

## 解決方法

- 修正: `Datetime::validate`（src/datetime.rs）を `pub fn` として新規実装する。各フィールドの `validate()` 呼び出しに加えて、4 バリアント以外の組み合わせ（date と time の両方が None、offset が Some で date か time の片方が None）を拒否する
- 修正: `Date::validate`（src/datetime.rs）に year ≤ 9999 のチェックを追加する
- 修正: `Serializer::write_inline_value`（src/serializer.rs）の `Value::Datetime` 分岐で、出力前に `dt.validate()` を呼び、`Error::Validate` を `Error::Serialize` に変換して返す
- テスト: シリアライザ経由の検証を `tests/test_serializer.rs` に、`validate` 単体の検証を `tests/test_datetime.rs` に、`Document` 経由の検証を `tests/test_edit.rs` に追加する。無効ケースは `to_string` と `to_inline_string` の 2 経路で検証する（`to_string_pretty` は `to_string` と共通実装のため省略可）
  - `tests/test_serializer.rs`: 無効な `Datetime` がエラー（`Error::Serialize`）になること
    - date / time 両方 None の `Datetime`（offset あり・なし両方）
    - 範囲外の `Date`（month 13 / day 40 / year 10000）・`Time`（hour 24）・`Offset`（minutes 1440）を含む `Datetime`
    - date のみ + offset、time のみ + offset の組み合わせ（オフセットが黙って破棄されないこと）
  - `tests/test_serializer.rs`: valid な 4 バリアントのラウンドトリップ（`to_string` でシリアライズして再パースし、値が一致すること、および単体の `Value::Datetime` を `to_inline_string` に渡して Ok になること）が従来どおり成功すること
  - `tests/test_edit.rs`: 無効な `Datetime` を `Document::set` / `set_path`（src/edit.rs）に渡すと `Err` になり、ドキュメントの内容が変化しないこと
  - `tests/test_datetime.rs`: `Date::validate` の year 境界値（10000 はエラー、9999 は成功）と、`Datetime::validate` の 4 バリアント以外の組み合わせがエラーになること
- `CHANGES.md` に `[FIX]` エントリを追加する
