RUSTC ?= rustc
RUSTFLAGS ?= -O

all: c64

run: c64
	./c64

test: c64_test
	./c64_test

clean:
	rm -rf *.dSYM *~ c64 c64_test

.PHONY: all run test clean

%: %.rc *.rs
	$(RUSTC) $(RUSTFLAGS) --bin -o $@ $<

%_test: %.rc *.rs
	$(RUSTC) $(RUSTFLAGS) --test -o $@ $<
