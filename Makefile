BINS = $(wildcard src/*.rs)
BIN_TGDS = $(patsubst src/%.rs,rootfs/bin/%,$(BINS))
build: $(BIN_TGDS)
# .PHONY: build
rootfs/bin/%: src/%.rs
	mbcc $<