# 0003: toml-test を気軽に実行できる仕組みを追加する

## タスク

`toml-test` をローカルで 1 コマンド実行できる導線を追加する。

## 実施内容

- `tools/toml-test-adapter` を追加し、`toml-test` 互換の decoder / encoder バイナリを実装。
- `scripts/run_toml_test.sh` を追加し、以下を自動化:
  - `toml-test` の clone (`.cache/toml-test`)
  - adapter の build
  - `go run ./cmd/toml-test test` の実行
- `Makefile` にターゲット追加:
  - `make toml-test`
  - `make toml-test-time`
- `README.md` に実行方法を追記。
- `Cargo.toml` の `workspace.exclude` に `tools/toml-test-adapter` を追加。

## 確認結果

- `make toml-test-time`: valid 9 / encoder 9 / invalid 70 がすべて成功。
- `make toml-test`: valid 205 / encoder 205 / invalid 473 がすべて成功。
- `make fmt`: 成功
- `make clippy`: 成功
- `make check`: 成功
- `make test`: 成功

