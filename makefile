PROJDIR=$(shell git rev-parse --show-toplevel)
VERSION=$(shell cat src/main.rs | grep version| grep -oE "[0-9]{1,2}.[0-9]{1,2}.[0-9]")

build:
	@cargo build

info:
	@cargo run --bin wikit -- dict --info test/demo.mdx

create-txt-from-mdx:
	@cargo run --bin wikit -- dict --create --output demo.txt output.mdx

create-mdx-from-txt:
	@cargo run --bin wikit -- mdx --create --output output.mdx test/demo.txt

test-create-mdx:
	@rm -rf test/demo.mdx && cargo test test_create_mdx -- --nocapture

test-parse-mdx: test-create-mdx
	@TEST_MDX_FILE=./test/demo.mdx cargo test test_parse_mdx -- --nocapture

clean:
	@rm -rf wikit-mac-*.zip wikit-linux-*.zip wikit-win-*.zip

image:
	@bash ${PROJDIR}/scripts/build.sh image

container:
	@bash ${PROJDIR}/scripts/build.sh container

publish:
	@bash ${PROJDIR}/scripts/build.sh publish

mac:
	@cargo run --bin wikit -- dict --create --output output.dictionary test/demo.txt
