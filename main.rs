#[desc = "C64 emulator"];
#[author = "Andreas Neuhaus <zargony@zargony.com>"];
#[crate_type = "bin"];

mod addressable;
mod c64;
mod memory;
mod mos65xx;
mod ram;
mod rom;

fn main () {
	let c64 = c64::C64::new();
	c64.run();
}