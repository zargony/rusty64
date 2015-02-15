//!
//! C64 emulator
//!

// General information on C64 : http://unusedino.de/ec64/technical/aay/c64/
// Useful emulator information: http://emudocs.org/?page=Commodore%2064
// C64 memory map overview: http://www.c64-wiki.com/index.php/Memory_Map
// Details about the PLA: http://www.c64-wiki.de/index.php/PLA_(C64-Chip)
// Even more PLA details: http://skoe.de/docs/c64-dissected/pla/c64_pla_dissected_r1.1_a4ss.pdf

#![feature(core)]
#![feature(env)]
#![feature(fs)]
#![feature(io)]
#![feature(path)]

#![warn(missing_docs, bad_style, unused, unused_extern_crates, unused_import_braces, unused_qualifications, unused_typecasts)]

#[macro_use]
extern crate log;
extern crate rand;

mod cpu;
mod mem;

#[cfg(not(test))]
fn main () {
    // TODO
}
