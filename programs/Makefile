INCLUDE_DIR := $(shell pwd)/include/
SPASM := spasm
SPASM_FLAGS := -T -I "$(INCLUDE_DIR)"

all: displaytest.8xp helloworld.8xp keyrepeat.8xp phoenix.8xp

.PHONY: clean

clean:
	rm -f *.8xp *.lst

%.8xp %.lst: %.asm
	$(SPASM) $(SPASM_FLAGS) $< $@

phoenix.8xp phoenix.lst: $(wildcard phoenix/*)
	$(SPASM) $(SPASM_FLAGS) -I phoenix/ phoenix/mainmir.asm phoenix.8xp
