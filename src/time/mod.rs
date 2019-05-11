pub type Timestamp = u32;

pub struct Clock {
    time: Timestamp,
}

impl Clock {
    pub fn new() -> Clock {
        return Clock { time: 0 };
    }

    pub fn now(&mut self) -> Timestamp {
        // TODO: Don't increment each time, consecutive calls might happen
        // within the same time period, thus return values should possibly be
        // equal.
        self.time = self.time + 1;
        return self.time;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_doesnt_panic() {
        let mut c = Clock::new();

        c.now();
    }
}
