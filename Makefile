RUSTC ?= rustc
RUSTFLAGS ?= -O --cfg ndebug

TARGETS := c64

LIBSDL2 := libsdl2-79c1f430-0.0.1.rlib

all: $(TARGETS)

$(TARGETS): %: build/%

run: build/$(firstword $(TARGETS))
	$<

check: build/test
	$<

clean:
	rm -rf build sdl2/build

distclean: clean
	$(MAKE) -C vendor/sdl2 clean

.PHONY: all $(TARGETS) run check clean distclean

$(patsubst %,build/%,$(TARGETS)): build/%: src/bin.rs build/$(LIBSDL2)
	mkdir -p build
	$(RUSTC) $(RUSTFLAGS) --dep-info build/$*.d -L build --cfg $* -o $@ $<

build/test: src/test.rs build/$(LIBSDL2)
	mkdir -p build
	$(RUSTC) $(RUSTFLAGS) --dep-info build/test.d -L build --test -o $@ $<

-include $(patsubst %,build/%.d,$(TARGETS))
-include build/test.d

build/$(LIBSDL2):
	mkdir -p build
	$(MAKE) -C vendor/sdl2 build/tmp/libsdl2.dummy
	cp vendor/sdl2/build/lib/$(LIBSDL2) build/
