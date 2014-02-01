RUSTC ?= rustc
RUSTFLAGS ?= -O --cfg ndebug

TARGETS := c64

LIBSDL2 := libsdl2-16412a49-0.0.1.rlib

all: $(TARGETS)

$(TARGETS): %: build/%

run: build/$(firstword $(TARGETS))
	$<

check: build/test
	$<

clean:
	rm -rf build sdl2/build

.PHONY: all $(TARGETS) run check clean

sdl2/build/lib/$(LIBSDL2):
	$(MAKE) -C sdl2 build/tmp/libsdl2.dummy

$(patsubst %,build/%,$(TARGETS)): build/%: src/bin.rs sdl2/build/lib/$(LIBSDL2)
	mkdir -p build
	$(RUSTC) $(RUSTFLAGS) --dep-info build/$*.d -L sdl2/build/lib --cfg $* --bin -o $@ $<

-include $(patsubst %,build/%.d,$(TARGETS))

build/test: src/test.rs
	mkdir -p build
	$(RUSTC) $(RUSTFLAGS) --dep-info --test -o $@ $<

-include build/test.d
