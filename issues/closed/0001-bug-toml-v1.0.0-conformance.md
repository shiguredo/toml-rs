# 0001: TOML v1.0.0 準拠性の不備

## 背景

`refs/v1.0.0.md` の以下の規定に対して、実装が許容しすぎている、または不正入力で停止しない可能性がある。

- dotted keys で定義されたテーブルは `[table]` で再定義できない
- 配列テーブル `[[table]]` と同名の通常テーブル `[table]` は定義できない
- Offset Date-Time は RFC 3339 形式（`T` と `Z`、`±HH:MM`）
- 改行は LF または CRLF

## 再現手順

### 1. dotted keys で定義したテーブルの再定義

```toml
[fruit]
apple.color = "red"

[fruit.apple] # v1.0.0 では不正
texture = "smooth"
```

### 2. 配列テーブルと同名の通常テーブル定義

```toml
[[fruits]]
name = "apple"

[fruits] # v1.0.0 では不正
color = "red"
```

### 3. RFC 3339 の厳密性

```toml
a = 1979-05-27t07:32:00Z      # 't' は不正
b = 1979-05-27T07:32:00z      # 'z' は不正
c = 1979-05-27T07:32:00+00:60 # 分が 60 は不正
```

### 4. 改行の不正（CR 単体）

```text
\r
```

## 期待する挙動

- v1.0.0 の規定に反する入力は parse error とする。
- 不正な改行文字を含む入力でも、無限ループせずに parse error とする。

## 対応方針

- dotted keys により生成した中間テーブルを「定義済み」として扱い、`[header]` による再定義をエラーにする。
- `[[header]]` で確立したパスに対して `[header]` をエラーにする。
- 日時パースで `T` / `Z` のみ許容し、`±HH:MM` の `MM` を検証する。
- CR 単体の改行を parse error とする。

## 対応内容

- dotted keys で生成した中間テーブルを `TableState::Dotted` として追跡し、`[header]` での再定義を拒否するようにした。
- 既に配列テーブルとして確立したパスに対する `[header]` を parse error にした。
- 日時の区切りを `T`、UTC を `Z` のみに限定し、オフセットの分 (`MM`) を 0..=59 に制限した。
- CR 単体の改行を parse error とし、無限ループしないようにした。

## 確認

- `make fmt`
- `make clippy`
- `make check`
- `make test`
