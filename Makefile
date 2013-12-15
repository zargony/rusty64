RUSTC ?= rustc
RUSTFLAGS ?= -O --cfg ndebug

TARGETS := $(filter-out test,$(patsubst src/%.rs,%,$(wildcard src/*.rs)))

all: $(TARGETS)

$(TARGETS): %: build/%

run: build/$(firstword $(TARGETS))
	$<

check: build/test
	$<

clean:
	rm -rf build

.PHONY: all $(TARGETS) run check clean

build:
	mkdir -p $@

$(patsubst %,build/%,$(TARGETS)): build/%: src/%.rs build
	$(RUSTC) $(RUSTFLAGS) --dep-info --bin -o $@ $<

-include $(patsubst %,build/%.d,$(TARGETS))

build/test: src/test.rs build
	$(RUSTC) $(RUSTFLAGS) --dep-info --test -o $@ $<

-include build/test.d
