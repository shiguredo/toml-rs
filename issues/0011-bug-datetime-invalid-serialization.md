# 無効な Datetime からシリアライザが Ok のまま無効な TOML を生成する

- Created: 2026-08-06
- Completed: {YYYY-MM-DD}
- Branch: feature/fix-datetime-serialization-validation
- Polished: {YYYY-MM-DD}

## 目的

公開 API 経由で構築された無効な `Datetime` / `Date` / `Time` に対して、シリアライザが `Ok` を返しながら再パース不能な TOML を生成したり、オフセット情報を黙って破棄したりする経路を塞ぐ。

## 現状

`Datetime` / `Date` / `Time` のフィールドはすべて `pub`（src/datetime.rs）で、無効な組み合わせを型システムで防げない。一方、シリアライズ経路では `validate()` が呼ばれないため、以下の問題が発生する:

1. `Datetime { date: None, time: None, offset: Some(...) }` は `Datetime` の `Display` 実装（src/datetime.rs）の `(None, None, _)` アームで空文字を出力し、`Serializer::write_inline_value`（src/serializer.rs）がそれをそのまま出力する。`to_string` / `to_inline_string` / `to_string_pretty` は `Ok` を返しながら `key = ` のような再パース不能な TOML を生成する
2. `Datetime { date: Some(...), time: None, offset: Some(...) }` や `Datetime { date: None, time: Some(...), offset: Some(...) }` は `Display` がオフセットを出力せず、`1979-05-27` のように情報が黙って失われる（データ損失）
3. 範囲外の値（例: `Date { month: 13, day: 40 }`）は `2024-13-40T01:02:03` のような再パース不能な TOML を `Ok` で生成する

パース経路では常に検証済みの値が生成されるため、いずれも利用者が `Datetime` 等を直接構築した場合にのみ発生する。

## 設計方針

シリアライズ前に `Datetime` / `Date` / `Time` の `validate()` を呼び、無効な値に対しては `Error::Serialize` を返す。`validate()` は各型に `pub fn` として既に実装されている（src/datetime.rs の `Date::validate` / `Time::validate` / `Offset::validate` / `Datetime::validate` 相当）。

`Display` 実装の `(None, None, _)` アームは TOML 上存在し得ない組み合わせであり、空文字出力は無効 TOML の温床のため、エラー表現（`fmt::Error`）にするか削除する。

## 完了条件

- 無効な `Datetime` / `Date` / `Time` を `Value::Datetime` に包んで `to_string` / `to_inline_string` / `to_string_pretty` に渡すと `Err` が返る
- シリアライザが `Ok` を返しながら再パース不能な TOML を生成する経路が存在しない
- パース経路で生成される値（valid な Datetime）のシリアライズは従来どおり成功する

## 解決方法

- 修正: `Serializer::write_inline_value`（src/serializer.rs）の `Value::Datetime` 分岐で、出力前に `validate()` を呼びエラーを返す
- 修正: `Datetime` の `Display` 実装（src/datetime.rs）の `(None, None, _)` アームを空文字出力ではなくエラー表現にする
- テスト: `tests/test_serializer.rs` に以下を追加する
  - `date` / `time` 両方 `None` の `Datetime` がエラーになること
  - 範囲外の `Date`（month 13 等）がエラーになること
  - 片方のみ + `offset` の組み合わせでオフセットが破棄されない（エラーになる）こと
  - valid な全バリアントのラウンドトリップが従来どおり成功すること
- `CHANGES.md` に `[FIX]` エントリを追加する
