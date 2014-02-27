This is an attempt to create an emulator platform for 16-bit computers in [Rust](http://www.rust-lang.org). It aims to emulate a [C64](http://en.wikipedia.org/wiki/Commodore_64) initially and maybe other computers some day.

The emulator consists of independent modules that emulate hardware pieces like generic RAM or a 6502 CPU. It's an interpreting emulation (no code translation at run time) and aims to be cycle-accurate. Hardware emulating modules are connected together, loaded with firmware and become an emulator for e.g. the C64.

I'm aiming to find a good balance between a nice hardware abstraction, idiomatic Rust programming, a correct emulation and a good emulation speed.

This a fun project I started a while ago to practice Rust development. It's far from being usable in any way. I'm planning to push it forward from time to time in my free time, but don't expect frequent updates :)
