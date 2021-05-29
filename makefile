.PHONY: test

test:
	@rm -rf test/demo.mdx && cargo test test_create_mdx -- --nocapture && hexdump -C test/demo.mdx
