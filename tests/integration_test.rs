extern crate id_generator;

mod simulator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_run() {
        let mut simulator = simulator::Simulator::new(2, 3, 1);


        simulator.run();

        let validation = simulator.validate_run();

        match validation {
            Ok(_) => {}
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn one_client_two_servers() {
        let mut simulator = simulator::Simulator::new(1, 2, 1);

        simulator.run();

        let validation = simulator.validate_run();

        match validation {
            Ok(_) => {}
            Err(e) => panic!(e),
        }
    }
}
