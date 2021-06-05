PROJDIR=$(shell git rev-parse --show-toplevel)
VERSION=$(shell cat src/main.rs | grep version| grep -oE "[0-9]{1,2}.[0-9]{1,2}.[0-9]")

info:
	@cargo run --bin wikit -- mdx --info test/demo.mdx

parse:
	@cargo run --bin wikit -- mdx --parse --output demo.txt output.mdx

create:
	@cargo run --bin wikit -- mdx --create --output output.mdx test/demo.txt

test-create:
	@rm -rf test/demo.mdx && cargo test test_create_mdx -- --nocapture

test-parse: test-create
	@TEST_MDX_FILE=./test/demo.mdx cargo test test_parse_mdx -- --nocapture

clean:
	@rm -rf wikit-mac-*.zip wikit-linux-*.zip wikit-win-*.zip

mac:
	@CC=o64-clang CXX=o64-clang++ cargo build --release --target x86_64-apple-darwin
	@cd target/x86_64-apple-darwin/release && zip -r wikit-mac-v${VERSION}.zip wikit && mv wikit-mac-v${VERSION}.zip ${PROJDIR}

linux:
	@cargo build --release --target x86_64-unknown-linux-gnu
	@cd target/x86_64-unknown-linux-gnu/release && zip -r wikit-linux-v${VERSION}.zip wikit && mv wikit-linux-v${VERSION}.zip ${PROJDIR}

win:
	@cargo build --release --target x86_64-pc-windows-gnu
	@cd target/x86_64-pc-windows-gnu/release && zip -r wikit-win-v${VERSION}.zip wikit.exe && mv wikit-win-v${VERSION}.zip ${PROJDIR}

release: mac linux win
