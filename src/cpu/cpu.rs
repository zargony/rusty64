//!
//! Generic CPU handling
//!

/// A generic trait for CPUs
pub trait CPU {
    /// Reset the CPU
    fn reset (&mut self);

    /// Do one step (execute the next instruction). Return the number of cycles that were
    /// simulated.
    fn step (&mut self) -> usize;
}
