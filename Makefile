.PHONY: fmt-ci
fmt-ci:
	npx prettier .github --write

.PHONY: build-release-windows
build-release-windows:
	cargo build -p rpgmv-image-viewer --release --target x86_64-pc-windows-msvc