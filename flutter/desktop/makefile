run:
	@cargo run

reload:
	@flutter attach -d linux --debug-url=$(shell cat ${XDG_RUNTIME_DIR}/vmservice.app_template | grep -Eo '"http://.*"')
