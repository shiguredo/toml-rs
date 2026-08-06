# 配列テーブルを再オープン後にサブテーブルを定義すると valid TOML が拒否される

- Created: 2026-08-06
- Completed: {YYYY-MM-DD}
- Branch: feature/fix-array-table-state-reset
- Polished: {YYYY-MM-DD}

## 目的

`[[a]]` で配列テーブルに新しい要素を追加した後、新要素配下にサブテーブルを定義する正当な TOML がパースエラーになるバグを修正する。

## 現状

`Parser::handle_array_table`（src/parser.rs）は `[[a]]` の再オープン時に `table_states` のみ `retain` でリセットし、`array_table_paths` と `array_table_current_index` は前要素時代の状態が残ったままになる。このため、新要素配下のパスに前要素で配列テーブルが定義されていた名前と同じ名前を使うと、stale な状態に引きずられて正当な入力が拒否される。

以下の入力は TOML 仕様（refs/v1.0.0.md の「Any reference to an array of tables points to the most recently defined table element of the array」）に照らし、また Python 標準ライブラリ tomllib でも valid と判定されるが、本ライブラリではパースエラーになる:

```toml
[[a]]
[a.b]
[[a.b.c]]
[a.b.c.d]
w = 1
[[a]]
[a.b.c.d]
```

この入力は `internal error: array table 'c' not found` を返す。`internal error` は設計上ユーザー入力で発生してはならない経路（`handle_array_table` / `handle_standard_table` の `ok_or_else`）であり、stale な `array_table_paths` が原因で発火する。

同根の別ケースとして、`[[a]]` 再オープン後に `[a.b.c]`（前要素で `[[a.b.c]]` が定義されていた名前）を標準テーブルとして定義する入力も `'a.b.c' is defined as an array table and cannot be a standard table` で拒否される。

## 設計方針

`[[a]]` で新しい要素を追加する際、新要素のスコープに入る配列テーブル状態（`array_table_paths` / `array_table_current_index`）を `table_states` と同様にリセットする。

`table_states` のリセットは「現在のパスより深いサブパス」を `retain` で削除する方式（`k.len() > path.len() && k.starts_with(&path)`）。`array_table_paths` / `array_table_current_index` にも同じ条件で適用するのが最小の変更となる。

代替案として、ナビゲーション時に `array_table_paths` を信用せず実体（テーブルツリーの内容）に従う方式もあるが、変更範囲が大きいため、まずは状態リセットの追加で対応する。

## 完了条件

- 上記の再現入力を含め、`[[a]]` 再オープン後に新要素配下へサブテーブル・配列テーブルを定義する入力がすべてパースできる
- ユーザー入力で `internal error` を含むエラーメッセージが発生しない
- 上記入力が TomlVersion::V1_0 / V1_1 の両方でパースできる

## 解決方法

- 修正: `Parser::handle_array_table` の `table_states.retain` と同じ条件で `array_table_paths` / `array_table_current_index` も `retain` する
- テスト: `tests/test_parser.rs` に以下を追加する
  - `[[a]]` 再オープン後の新要素へのサブテーブル定義（再現入力）
  - `[[a]]` 再オープン後に前要素の配列テーブル名を標準テーブルとして再定義
  - `[[a]]` 再オープン後に前要素と同じ配列テーブル名を再度 `[[...]]` で定義（新要素では未定義扱いになることの確認）
  - 同ケースを TomlVersion::V1_1 でも検証
- `CHANGES.md` に `[FIX]` エントリを追加する
