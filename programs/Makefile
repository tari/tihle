INCLUDE_DIR := $(shell pwd)/include/
SPASM := spasm
SPASM_FLAGS := -T -I "$(INCLUDE_DIR)"

AUTO_8XPS := $(addsuffix .8xp,$(basename $(wildcard *.asm)))

all: phoenix.8xp $(AUTO_8XPS)

.PHONY: clean

clean:
	rm -f *.8xp *.lst

%.8xp %.lst: %.asm
	$(SPASM) $(SPASM_FLAGS) $< $@

phoenix.8xp phoenix.lst: $(wildcard phoenix/*)
	$(SPASM) $(SPASM_FLAGS) -I phoenix/ phoenix/mainmir.asm phoenix.8xp
