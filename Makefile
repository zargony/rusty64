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

%: %.rs *.rs
	$(RUSTC) $(RUSTFLAGS) --bin -o $@ $<

lib%.dylib: %.rs *.rs
	$(RUSTC) $(RUSTFLAGS) --lib -o $@ $<

%_test: %.rs *.rs
	$(RUSTC) $(RUSTFLAGS) --test -o $@ $<
