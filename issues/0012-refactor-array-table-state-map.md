# 配列テーブル状態を 2 つのフィールドから 1 つのマップに統合する

- Created: 2026-08-06
- Completed: {YYYY-MM-DD}
- Branch: feature/refactor-array-table-state-map
- Polished: {YYYY-MM-DD}

## 目的

`Parser`（src/parser.rs）の配列テーブル状態を保持する `array_table_paths`（`BTreeSet<Vec<String>>`）と `array_table_current_index`（`BTreeMap<Vec<String>, usize>`）はキー集合が常に一致する冗長な構造で、同期を維持する義務が挿入・削除の各所に散らばっている。これを 1 つの `BTreeMap<Vec<String>, usize>` に統合し、同期義務とパニック経路を無くす。

## 現状

- `Parser`（src/parser.rs）は配列テーブルとして定義されたパスの集合（`array_table_paths`）と、パスごとの現在要素インデックス（`array_table_current_index`）を別々のフィールドで保持している
- 両フィールドは挿入（`handle_array_table` 内の 3 箇所）と削除（`handle_array_table` 内の retain 2 箇所）で常にペアで更新されており、キー集合は常に一致する
- 参照側の `current_context_path` は `array_table_paths.contains(&prefix)` をガードに `array_table_current_index` を引いており、`expect("array_table_paths and array_table_current_index must be in sync")` のパニック経路を持つ
- 配列テーブルの再オープン時（配列テーブル状態のリセット）にも、2 フィールドに同じ retain 条件を適用して同期を保つ必要があり、変更のたびに同期義務の検討が必要になる

## 設計方針

`array_table_paths` と `array_table_current_index` を 1 つの `BTreeMap<Vec<String>, usize>` に統合する。

- パスの集合としての判定（`contains`）は `contains_key` で代替する
- パスの列挙が必要な箇所（`current_context_path` など）は `keys()` を利用する
- `navigate_table_mut` の引数（src/parser.rs）も統合後の型に合わせる
- 統合により「2 フィールドの同期」という不変条件自体が消え、`must be in sync` の `expect` も不要になる

## 完了条件

- `Parser` の `array_table_paths` / `array_table_current_index` の 2 フィールドが 1 つの `BTreeMap<Vec<String>, usize>` に統合される
- `current_context_path` の `expect("array_table_paths and array_table_current_index must be in sync")` パニック経路が無くなる
- `navigate_table_mut` のシグネチャが統合後の型に追随する
- パース結果に変化がなく、既存の全テスト（配列テーブル関連の `array_table_state_reset` 等を含む）が通る

## 解決方法

- `Parser` のフィールド定義と初期化を 1 つの `BTreeMap<Vec<String>, usize>` にまとめる
- `handle_array_table` / `handle_standard_table` / `current_context_path` / `navigate_table_mut` の `contains` を `contains_key` に置き換える
- 挿入・retain のペア更新を 1 つの操作にまとめる
- 不要になった同期コメントを削除する
- パース結果が変わらないことを既存テストで検証する（新規テストは不要。挙動を変えないリファクタリングのため）
