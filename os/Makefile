SPASM ?= spasm

BINARIES := $(addsuffix .bin,$(basename $(wildcard page*.asm)))

all: $(BINARIES)

%.bin %.lst: %.asm
	$(SPASM) -T -I ../programs/include/ $< $@

.PHONY: clean

clean:
	rm -f $(BINARIES) $(addsuffix .lst,$(basename $(BINARIES)))
