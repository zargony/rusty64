RUSTC ?= rustc
RUSTFLAGS ?= -O

all: c64

run: c64
	./c64

check: c64_test
	./c64_test

clean:
	rm -rf *.dSYM *~ c64 c64_test

.PHONY: all run check clean

c64: c64.rs *.rs
	$(RUSTC) $(RUSTFLAGS) --bin -o $@ $<

c64_test: c64.rs *.rs
	$(RUSTC) $(RUSTFLAGS) --test -o $@ $<
