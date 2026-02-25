.PHONY: test cover pbt pbt-cover fuzz fuzzing fuzzing-parallel fuzzing-list check clippy fmt clean toml-test toml-test-v1_1 toml-test-time toml-test-ci

# 全テストを実行する
test:
	cargo test --workspace

# 全テストカバレッジ付きで実行する
cover:
	cargo llvm-cov --tests --workspace

# PBT をカバレッジ付きで実行する
pbt-with-cover:
	cargo llvm-cov -p pbt --tests

# Fuzzing を全ターゲットで逐次実行する（fork 数はコア数に応じて自動調整）
fuzzing:
	@FORKS=$$(( $$(nproc) - 2 )); \
	if [ $$FORKS -lt 1 ]; then FORKS=1; fi; \
	echo "Using fork=$$FORKS on $$(nproc) cores"; \
	for target in $$(cargo fuzz list); do \
		echo "=== Fuzzing $$target ==="; \
		cargo +nightly fuzz run $$target -- -max_total_time=30 -fork=$$FORKS -max_len=4096 || exit 1; \
	done

# Fuzzing を全ターゲットで並列実行しレポートを出力する（fork 数はコア数に応じて自動調整）
fuzzing-parallel:
	@mkdir -p fuzz/logs
	@FORKS=$$(( $$(nproc) - 2 )); \
	if [ $$FORKS -lt 1 ]; then FORKS=1; fi; \
	echo "Using fork=$$FORKS on $$(nproc) cores"; \
	cargo fuzz list | xargs -P $$(cargo fuzz list | wc -l) -I {} \
		sh -c 'cargo +nightly fuzz run {} -- -max_total_time=30 -fork=1 -max_len=4096 > fuzz/logs/{}.log 2>&1'
	@echo "=== Fuzzing Report ==="
	@for f in fuzz/logs/*.log; do \
		target=$$(basename $$f .log); \
		last=$$(grep -E '^#[0-9]+:' $$f | tail -1); \
		echo "$$target: $$last"; \
	done

# Fuzzing ターゲット一覧を表示する
fuzzing-list:
	cargo fuzz list

# cargo check を実行する
check:
	cargo check --workspace

# cargo clippy を実行する
clippy:
	cargo clippy --workspace -- -D warnings

# cargo fmt を実行する
fmt:
	cargo fmt --all

# toml-test (TOML v1.0) を実行する
toml-test:
	./scripts/run_toml_test.sh

# toml-test (TOML v1.1) を実行する
toml-test-v1_1:
	TOML_VERSION=1.1 ./scripts/run_toml_test.sh

# toml-test の日時系テストのみを実行する
toml-test-time:
	./scripts/run_toml_test.sh -run 'valid/datetime/*,valid/local-time/*,valid/local-date/*,valid/local-datetime/*,invalid/datetime/*,invalid/local-time/*,invalid/local-date/*,invalid/local-datetime/*,encoder/datetime/*,encoder/local-time/*,encoder/local-date/*,encoder/local-datetime/*'

# CI 向けに toml-test を実行する
toml-test-ci:
	TOML_TEST_UPDATE=1 ./scripts/run_toml_test.sh -parallel 4 -color never

# ビルド成果物を削除する
clean:
	cargo clean
