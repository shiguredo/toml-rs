# 0004: toml-test 用 CI ターゲットを ubuntu-slim で動かす

## タスク

`toml-test` を CI で実行できるようにし、`ubuntu-slim` で動作させる。

## 実施内容

- `Makefile` に CI 用ターゲット `toml-test-ci` を追加。
  - `TOML_TEST_UPDATE=1`
  - `-parallel 4`
  - `-color never`
- `.github/workflows/ci.yml` に `toml_test` ジョブを追加。
  - `runs-on: ubuntu-slim`
  - `actions/checkout@v6`
  - `dtolnay/rust-toolchain@stable`
  - `actions/setup-go@v6 (go-version: 1.25)`
  - `make toml-test-ci` を実行
- `slack_notify` の `needs` に `toml_test` を追加。

## 確認結果

- `make toml-test-ci`: valid 205 / encoder 205 / invalid 473 がすべて成功
- `make check`: 成功
