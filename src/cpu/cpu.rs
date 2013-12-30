use mem::{Addr, Addressable};

/// A generic trait for CPUs
pub trait CPU<A: Addr> {
	/// Reset the CPU
	fn reset<M: Addressable<A>> (&mut self, mem: &M);

	/// Do one step (execute the next instruction). Returns the number of
	/// cycles the instruction needed
	fn step<M: Addressable<A>> (&mut self, mem: &mut M) -> uint;
}
