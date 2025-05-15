.PHONY: fmt-ci
fmt-ci:
	npx prettier .github --write

.PHONY: build-release-windows
build-release-windows:
	cargo build -p rpgmv-image-viewer -p rpgmv-tool --release --target x86_64-pc-windows-msvc

.PHONY:	rm-nightly-release
rm-nightly-release:
	nightly_releases_count=$$(gh release list --json name | jq '[.[] | select(.name == "nightly")] | length'); \
	echo Located $$nightly_releases_count releases;                                                            \
	if [[ $$nightly_releases_count != "0" ]]; then                                                             \
		echo Deleting release;                                                                                 \
		gh release delete nightly -y --cleanup-tag;                                                            \
	fi