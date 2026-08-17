# PBT の依存を proptest から noprop に切り替える

- Created: 2026-08-17
- Completed: {YYYY-MM-DD}
- Branch: feature/change-proptest-to-noprop
- Polished: 2026-08-17

## 目的

`pbt/` のプロパティベーステストが依存している `proptest` を、crates.io の `noprop` に切り替える。

`noprop` は依存ゼロ・マクロなし・`unsafe` なしで、シードとケース数を呼び出し側が明示する。`shiguredo-rust` スキルの「依存は最小限にすること」の方針に合い、`proptest` と、`proptest` 経由でのみ引き込まれる依存（`rand` / `rusty-fork` 等）を排除できる。シードとケース数をテストコード上で明示するため、失敗再現の手段も読み取りやすい。

## 現状

- `pbt/Cargo.toml` が `proptest = "1.11"` を依存に持つ
- `pbt/src/lib.rs` の生成ヘルパー `bare_key_strategy` / `date_strategy` / `time_strategy` / `offset_strategy` / `datetime_strategy` / `safe_string_strategy` / `value_strategy` / `table_strategy` は `proptest::prelude::*` の `Strategy` / `prop_map` / `prop_oneof!` / `prop_recursive` 等に依存している
- テストは次の 5 ファイルが `proptest!` マクロを使う（`prop_assert*` は 5 ファイルすべて、`prop_assume!` は `pbt/tests/prop_edit.rs` が使用）
  - `pbt/tests/prop_parser.rs`
  - `pbt/tests/prop_serializer.rs`
  - `pbt/tests/prop_datetime.rs`
  - `pbt/tests/prop_value.rs`
  - `pbt/tests/prop_edit.rs`
- `skills/shiguredo-toml/SKILL.md` も `proptest` 前提の説明になっている

## 設計方針

- 時雨堂共有の `shiguredo-rust` スキルには「PBT は proptest を使うこと」の規約があるが、本 issue はこれと衝突する。スキルは本リポジトリ外（llm-feedback で管理）のため本 issue の変更対象に含めず、切り替え完了後に llm-feedback 側でスキルの規約を更新する（本 issue の完了条件には含めない）
- 依存は crates.io の `noprop` を使う（バージョンは `noprop = "0.2"` のようにマイナーバージョンまで指定する）
- `proptest` の combinator DSL / マクロを残さず、`noprop::Runner` と `sample_*` による imperative な API に置き換える
- 既存 PBT が検証しているプロパティ（ラウンドトリップ、パース不変条件、Datetime の妥当性、編集後の一貫性など）の意図は維持する
- 生成ロジックは `pbt/src/lib.rs` に `sample_*` 関数として集約し、各 `prop_*.rs` から呼ぶ形にする
- シードは `noprop::seed_from_env_or_time` で取得し、環境変数名は `NOPROP_SEED` に統一する。ケース数（case budget）はテストごとに明示し、既存の proptest デフォルトと同じ 256 とする
- `noprop` は自動 shrinking を持たない前提で設計する。失敗入力はシード再現と生成トレースで特定し、必要なら通常の回帰テストへ落とす

## 完了条件

- `pbt/Cargo.toml` から `proptest` が消え、`Cargo.lock` にも `proptest` が残らず、`noprop` のみで PBT がビルド・実行できる
- `pbt/src/lib.rs` と `pbt/tests/prop_*.rs` に `proptest` の import・マクロ・Strategy API が残っていない
- 既存と同等のプロパティ検証が `cargo test -p pbt` で通る
- `skills/shiguredo-toml/SKILL.md` の PBT 説明が `noprop` 前提に更新されている
- `CHANGES.md` に切り替えのエントリが追記されている

## 解決方法

- `pbt/Cargo.toml` の依存を `proptest` から `noprop` に差し替える
- `pbt/src/lib.rs` の Strategy 群を、`noprop` の `&mut TestCaseContext` と `sample_*` を受け取る生成関数へ書き換える
  - 日付の日数上限や Datetime バリアント選択など、以前 `prop_flat_map` / `prop_oneof!` で表現していた依存関係は通常の制御フローで書く
  - 再帰的な `Value` 生成は深さ・サイズ上限を明示した関数で書く（`prop_recursive` の代替）
- 各 `prop_*.rs` を `#[test] fn ... -> noprop::TestResult` + `Runner::new(seed).run(...)` 形式に書き換える
  - `prop_assert*` は通常の `assert!` / `assert_eq!` に置き換える
  - `prop_assume!` に相当する棄却は `noprop` の rejection 手段（または生成側で無効ケースを出さない）で扱う
- `skills/shiguredo-toml/SKILL.md` の `proptest` 記述を `noprop` に更新する
- `CHANGES.md` に切り替えのエントリを追記する（種別は `shiguredo-changelog` スキルに従う）
- `cargo test -p pbt` で全 PBT が通ることを確認する
