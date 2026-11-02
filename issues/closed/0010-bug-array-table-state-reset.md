# 配列テーブルを再オープン後にサブテーブルを定義すると valid TOML が拒否される

- Created: 2026-08-06
- Completed: 2026-08-06
- Branch: feature/fix-array-table-state-reset
- Polished: 2026-08-06

## 目的

`[[a]]` で配列テーブルに新しい要素を追加した後、新要素配下にサブテーブルを定義する正当な TOML がパースエラーになるバグを修正する。

## 現状

`Parser::handle_array_table`（src/parser.rs）は `[[a]]` の再オープン時に `table_states` のみ `retain` でリセットし、`array_table_paths` / `array_table_current_index` はサブパスのエントリ（前要素で定義された配列テーブルのパス）が残ったままになる。このため、新要素配下では実在しないサブパスを経由する定義（例: 前要素で `[[a.b.c]]` を定義した後に `[[a]]` で新要素を追加し、新要素配下に `[a.b.c.d]` を定義する入力）が stale な状態に引きずられて拒否される。

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

この入力は `internal error: array table 'c' not found` を返す。`internal error` は設計上ユーザー入力で発生してはならない経路（`handle_standard_table` / `handle_array_table` / `navigate_table_mut` の `ok_or_else`）であり、stale な `array_table_paths` が原因で発火する。

同根の別ケースとして、`[[a]]` 再オープン後に前要素で `[[a.b.c]]` が定義されていた名前を標準テーブルとして再定義する以下の入力も拒否される:

```toml
[[a]]
[a.b]
[[a.b.c]]
[a.b.c.d]
w = 1
[[a]]
[a.b.c]
```

この入力は `'a.b.c' is defined as an array table and cannot be a standard table` で拒否される。前要素の構成によっては（例: 前要素に `[[a.b]]` が定義されている場合）stale な途中パスを経由して `internal error: array table 'b' not found` になる。

## 設計方針

`[[a]]` で新しい要素を追加する際、新要素のスコープに入る配列テーブル状態（`array_table_paths` / `array_table_current_index`）を `table_states` と同様にリセットする。

`table_states` のリセットは「現在のパスより深いサブパス」を `retain` で削除する方式（`k.len() > path.len() && k.starts_with(&path)`）。`array_table_paths` / `array_table_current_index` にも同じ条件で適用するのが最小の変更となる。両フィールドは常にペアで更新されるため、同じ条件で retain すれば同期は保たれる。

配列テーブルの挙動は TOML v1.0.0 と v1.1.0 で共通（refs/v1.1.0.md にも同旨の記述がある）ため、バージョンによる分岐は不要である。

代替案として、ナビゲーション時に `array_table_paths` を信用せず実体（テーブルツリーの内容）に従う方式もあるが、変更範囲が大きいため、まずは状態リセットの追加で対応する。

## 完了条件

- 解決方法に列挙したテスト入力（再現入力・同根ケース・配列テーブルの新規定義・再定義）がすべてパースできる
- 上記のテスト入力で `internal error` を含むエラーメッセージが発生しない
- 上記のテスト入力が TomlVersion::V1_0 / V1_1 の両方でパースできる

## 解決方法

- 修正: `Parser::handle_array_table`（src/parser.rs）の `table_states.retain` と同じ条件（`k.len() > path.len() && k.starts_with(&path)`）で `array_table_paths` / `array_table_current_index` も `retain` する。新要素のスコープで前要素のサブパス定義を引き継がないことで、stale なパスによる誤ったエラーや `internal error` の発火を防ぐ
- テスト: `tests/test_parser.rs` の `array_table_state_reset` モジュールに以下を追加する（各テストは TomlVersion::V1_0 / V1_1 の両方で検証する）
  1. `subtable_under_new_element_after_reopen`（再現入力。修正前は `internal error: array table 'c' not found`）。`a` が 2 要素になり、2 番目の要素配下に `b.c.d` が標準テーブルとして定義されること、前要素の `w = 1` が引き継がれないことを検証
  2. `standard_table_replacing_previous_array_table`（同根ケース。修正前は `'a.b.c' is defined as an array table and cannot be a standard table`）。2 番目の要素配下の `b.c` が標準テーブル（配列ではない）になること、前要素の `d.w` が引き継がれないことを検証
  3. `new_array_table_under_reopened_element`（前要素に `[[a.b]]` が定義済みの構成で新要素配下に配列テーブルを新規定義。修正前は `internal error: array table 'b' not found`）。2 番目の要素配下の `b` が標準テーブル、`b.c` が配列テーブルとして新規作成されることを検証
  4. `same_array_table_name_under_new_element`（回帰テスト。修正前からパース成功する仕様準拠の挙動）。2 番目の要素配下の `b` が前要素とは別の新規配列になることを検証
  5. `standard_table_using_previous_array_table_as_parent`（バリアント。修正前は `internal error: array table 'b' not found`）。2 番目の要素配下の `b.c` が標準テーブルとして定義されることを検証
  6. `nested_array_table_reopen`（中段の配列テーブル `[[a.b]]` の再オープン。レビューで追加）。1 番目の要素の `b.c` が配列テーブルのまま、2 番目の要素の `b.c` が標準テーブルになることを検証
- `CHANGES.md` に `[FIX]` エントリを追加する
