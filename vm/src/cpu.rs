#[derive(Debug)]
pub struct Cpu {
    /// Holds the address of the next instruction to execute
    program_counter: usize,
}

impl Cpu {
    pub fn new(start_addr: usize) -> Self {
        Self {
            program_counter: start_addr,
        }
    }
}
