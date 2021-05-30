.PHONY: test

test:
	@rm -rf test/demo.mdx && \
	 cargo test test_create_mdx -- --nocapture

decode: test
	 TEST_MDX_FILE=./test/demo.mdx cargo test test_parse_mdx -- --nocapture
