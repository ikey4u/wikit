PROJDIR=$(shell git rev-parse --show-toplevel)
VERSION=$(shell cat src/main.rs | grep version| grep -oE "[0-9]{1,2}.[0-9]{1,2}.[0-9]")

build:
	@cargo build

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

publish:
	@bash ${PROJDIR}/scripts/build.sh publish