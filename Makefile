RUSTC ?= rustc
RUSTFLAGS ?= -O

# FIXME: Nothing to build yet, so we run tests by default for now
all: test

run: c64
	./c64

test: c64_test
	RUST_THREADS=1 RUST_LOG=c64 ./c64_test

clean:
	rm -rf *.dSYM *~ c64 c64_test

.PHONY: all run test clean

%: %.rc *.rs
	$(RUSTC) $(RUSTFLAGS) --bin -o $@ $<

%_test: %.rc *.rs
	$(RUSTC) $(RUSTFLAGS) --test -o $@ $<
