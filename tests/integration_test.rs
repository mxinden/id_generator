extern crate queues;
extern crate id_generator;

mod simulator;

use std::collections::HashMap;
use queues::*;
use id_generator::{Msg, Envelope, Client, Server};


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_run() {
        let mut in_flight: Queue<Envelope> = queue![];
        let mut clients = HashMap::new();
        let mut servers = HashMap::new();

        let client_addresses = vec!["client1".to_string(), "client2".to_string()];
        let server_addresses = vec!["server1".to_string(), "server2".to_string(), "server3".to_string()];

        for addr in client_addresses {
            clients.insert(addr.clone(), Client {
                addr: addr.clone(),
                claimed_ids: vec![],
                servers: server_addresses.clone(),
                highest_id_seen: 0,
                responses: HashMap::new(),
            });

            match in_flight.add(Envelope {
                from: "simulator".to_string(),
                to: addr,
                msg: Msg::StartRequest,
                time: 1,
            }) {
                Ok(_) => {},
                Err(e) => panic!(e.to_string()),
            };
        }

        for addr in server_addresses {
            servers.insert(addr, Server {
                addr: "server1".to_string(),
                highest_id_seen: 0,
            });
        }

        let mut simulator = simulator::Simulator {
            in_flight,
            clients,
            servers,
            goal_per_client: 1,
        };

        simulator.run();

        let validation = simulator.validate_run();

        match validation {
            Ok(_) => {}
            Err(e) => panic!(e),
        }
    }
}
