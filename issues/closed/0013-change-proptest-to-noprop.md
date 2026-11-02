# PBT の依存を proptest から noprop に切り替える

- Created: 2026-08-17
- Completed: 2026-08-17
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

- 変更: `pbt/Cargo.toml` の依存を `proptest = "1.11"` から `noprop = "0.2"` に差し替える。`Cargo.lock` から `proptest` とその推移的依存（`rand` / `rusty-fork` / `bit-set` 等）が除去された
- 変更: `pbt/src/lib.rs` の 8 つの Strategy 群を、`&mut noprop::TestCaseContext` を受け取る `sample_*` 関数（`sample_bare_key` / `sample_date` / `sample_time` / `sample_offset` / `sample_datetime` / `sample_safe_string` / `sample_value` / `sample_table`）に書き換える
  - 日付の月別日数上限（うるう年対応）や Datetime の 4 バリアント選択は、以前 `prop_flat_map` / `prop_oneof!` で表現していた依存関係を通常の制御フローで実装する
  - 再帰的な `Value` 生成は深さ上限 3 を明示した `sample_value_recursive` で実装する（`prop_recursive` の代替）
- 変更: 5 つの `pbt/tests/prop_*.rs` を `#[test] fn ... -> noprop::TestResult` + `Runner::new(seed).run(256, ...)` 形式に書き換える。`prop_assert*` は `assert!` / `assert_eq!` に、`prop_assume!` は生成側の制約（`sample_key_pair` による既存キーと新規キーの不一致保証）に置き換える。シードは全テストで `noprop::seed_from_env_or_time("NOPROP_SEED")` を使い、ケース数は 256 に統一する（`empty_table_roundtrip` は決定的検証のため 1 ケース）
- 変更: `skills/shiguredo-toml/SKILL.md` の `pbt/` の説明を `noprop` による検証に更新する
- 変更: `CHANGES.md` の `## develop` に `[CHANGE]` エントリを追加し、陳腐化した misc の「proptest を 1.11 に更新する」エントリを削除する
- テスト: 既存の PBT 20 テスト（ラウンドトリップ、パース不変条件、Datetime の妥当性、編集後の一貫性）をすべて noprop 形式に書き換え、`cargo test -p pbt` / `cargo test --workspace` / `cargo clippy --workspace --all-targets -- -D warnings` で全通過を確認する
