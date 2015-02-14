/// A generic trait for CPUs
pub trait CPU {
    /// Reset the CPU
    fn reset (&mut self);

    /// Do one step (execute the next instruction). Returns the number of
    /// cycles the instruction needed
    fn step (&mut self) -> uint;
}
