pub mod time;

pub type ID = u64;
pub type MachineID = u16;
pub type SeqNumber = u16;

pub trait Generator {
    fn generate(&mut self) -> ID;
}

pub struct BasicGenerator {
    clock: time::Clock,
    machine_id: MachineID,
    seq_number: SeqNumber,
}

impl BasicGenerator {
    pub fn new(machine_id: MachineID, clock: time::Clock) -> BasicGenerator {
        let seq_number = 0;
        return BasicGenerator {
            clock,
            seq_number,
            machine_id,
        };
    }
}

impl Generator for BasicGenerator {
    fn generate(&mut self) -> ID {
        self.seq_number = self.seq_number + 1;
        return ((self.clock.now() as u64) << 32)
            + ((self.machine_id as u64) << 16)
            + (self.seq_number as u64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_returns_something() {
        let mut g = BasicGenerator::new(0, time::Clock::new());
        g.generate();
    }

    #[test]
    fn it_returns_increasing_values() {
        let mut g = BasicGenerator::new(0, time::Clock::new());

        let (first, second) = (g.generate(), g.generate());

        assert!(first < second)
    }
}
