use quickcheck::{quickcheck, TestResult};

mod simulator;

#[test]
fn basic_run() {
    let mut simulator = simulator::Simulator::new(2, 2, 2, 2);

    simulator.run().unwrap();

    simulator.validate_run().unwrap();
}

#[test]
fn one_client_two_servers() {
    let mut simulator = simulator::Simulator::new(1, 2, 1, 2);

    simulator.run().unwrap();

    simulator.validate_run().unwrap();
}

#[test]
fn previous_failures() {
    struct Test {
        num_clients: usize,
        num_servers: usize,
        num_ids: usize,
        delay: u64,
    };

    let tests = vec![
        // This did exceed the artificial maximum time limit inside the
        // simulator.
        // Test {
        //     num_clients: 4,
        //     num_servers: 50,
        //     num_ids: 1,
        //     delay: 1,
        // },
        // 'Error: expected client-1 to claim 4 ids but got 6'.
        Test {
            num_clients: 1,
            num_servers: 3,
            num_ids: 4,
            delay: 1,
        },
    ];

    for test in tests {
        let mut simulator =
            simulator::Simulator::new(test.num_clients, test.num_servers, test.num_ids, test.delay);

        simulator.run().unwrap();

        simulator.validate_run().unwrap();

    }
}

quickcheck! {
    fn prop(clients: usize, servers: usize, ids: usize, delay: u64) -> TestResult {
        if clients < 1 || servers < 1 || ids < 1 || delay < 1 {
            return TestResult::discard();
        }

        if clients > 100 || servers > 100 || ids > 1000 || delay > 1000 {
            return TestResult::discard();
        }

        let mut simulator = simulator::Simulator::new(clients, servers, ids, delay);

        match simulator.run() {
            Ok(_) => {}
            Err(e) => return TestResult::error(e),
        };

        let validation = simulator.validate_run();

        match validation {
            Ok(_) => TestResult::passed(),
            Err(e) => TestResult::error(e),
        }
    }
}
