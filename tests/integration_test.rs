extern crate id_generator;
extern crate rand;

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

use quickcheck::{TestResult, quickcheck};

mod simulator;

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn basic_run() {
        let mut simulator = simulator::Simulator::new(2,2,2, 2);

        simulator.run();

        let validation = simulator.validate_run();

        match validation {
            Ok(_) => {}
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn one_client_two_servers() {
        let mut simulator = simulator::Simulator::new(1, 2, 1, 2);

        simulator.run();

        let validation = simulator.validate_run();

        match validation {
            Ok(_) => {}
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn random() {
        let mut rng = rand::thread_rng();

        let simulator = simulator::Simulator::new(
            rng.gen_range(0, 10),
            rng.gen_range(0, 10),
            rng.gen_range(0, 10),
            rng.gen_range(0, 10),
        );

        let validation = simulator.validate_run();

        match validation {
            Ok(_) => {}
            Err(e) => panic!(e),
        }
    }

    #[quickcheck]
    fn prop(x: usize, y: usize, z: usize, d: usize) -> TestResult{
        if y < 1 || z < 1 || d < 1 {
            println!("discarding y: {}, z: {}, d: {}", y, z, d);
            return TestResult::discard()
        }

        println!("x: {}, y: {}, z: {}, d: {}", x, y, z, d);
        let mut simulator = simulator::Simulator::new(x, y, z, d);

        simulator.run().map_err(|e| {
            println!("failed with: {}", e);
            return TestResult::failed();
        });

        let validation = simulator.validate_run();

        TestResult::from_bool(match validation {
            Ok(_) => true,
            Err(e) => {
                println!("error: {}", e);
                false
            },
        })
    }
}
