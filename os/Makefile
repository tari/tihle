SPASM ?= spasm

BINARIES := $(addsuffix .bin,$(basename $(wildcard page*.asm)))

all: $(BINARIES)

%.bin: %.asm
	$(SPASM) $< $@

.PHONY: clean

clean:
	rm -f $(BINARIES)
