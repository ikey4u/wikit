test-create:
	@rm -rf test/demo.mdx && cargo test test_create_mdx -- --nocapture

test-parse: test-create
	 TEST_MDX_FILE=./test/demo.mdx cargo test test_parse_mdx -- --nocapture
