//! C64 emulator

// General information on C64 : http://unusedino.de/ec64/technical/aay/c64/
// Useful emulator information: http://emudocs.org/?page=Commodore%2064
// C64 memory map overview: http://www.c64-wiki.com/index.php/Memory_Map
// Details about the PLA: http://www.c64-wiki.de/index.php/PLA_(C64-Chip)
// Even more PLA details: http://skoe.de/docs/c64-dissected/pla/c64_pla_dissected_r1.1_a4ss.pdf

#![warn(missing_docs, unused)]
#![allow(dead_code)]

mod addr;
mod cpu;
mod mem;

#[cfg(not(test))]
fn main() {
    env_logger::init();

    let _foo = cpu::Mos6510::new(mem::Ram::new());
}
