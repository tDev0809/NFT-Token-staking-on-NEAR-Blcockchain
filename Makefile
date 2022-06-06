RFLAGS="-C link-arg=-s"

release:
	$(call docker_build,_rust_setup.sh)
	mkdir -p release
	cp target/wasm32-unknown-unknown/release/ref_farming.wasm release/ref_farming_release.wasm
	cp release/ref_farming_release.wasm out/main.wasm

define docker_build
	docker build -t my-contract-builder .
	docker run \
		--mount type=bind,source=${PWD},target=/host \
		--cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
		-w /host \
		-e RUSTFLAGS=$(RFLAGS) \
		-i -t my-contract-builder \
		/bin/bash $(1)
endef
