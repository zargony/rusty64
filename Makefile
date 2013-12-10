RUSTC ?= rustc
RUSTFLAGS ?= -O --cfg ndebug

TARGETS := $(filter-out test,$(patsubst src/%.rs,%,$(wildcard src/*.rs)))
SOURCES := $(wildcard src/**/*.rs)

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

$(patsubst %,build/%,$(TARGETS)): build/%: src/%.rs $(SOURCES) build
	$(RUSTC) $(RUSTFLAGS) --bin -o $@ $<

build/test: src/test.rs $(SOURCES) build
	$(RUSTC) $(RUSTFLAGS) --test -o $@ $<
