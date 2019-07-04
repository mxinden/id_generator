extern crate id_generator;
extern crate queues;

use id_generator::{Addr, Client, Envelope, Msg, Receiver, Server, Timestamp};
use rand::{Rng, SeedableRng};
use std::collections::HashMap;

pub struct Simulator {
    pub in_flight: Vec<Envelope>,
    pub clients: HashMap<Addr, Client>,
    pub servers: HashMap<Addr, Server>,
    pub goal_per_client: usize,
    rng: rand::rngs::StdRng,
    time: Timestamp,
}

impl Simulator {
    pub fn new(
        num_clients: usize,
        num_servers: usize,
        num_ids_per_client: usize,
        network_seed: u64,
    ) -> Simulator {
        let in_flight: Vec<Envelope> = vec![];
        let mut clients = HashMap::new();
        let mut servers = HashMap::new();

        let mut client_addresses = vec![];
        let mut server_addresses = vec![];

        for i in 1..num_clients + 1 {
            client_addresses.push(format!("client-{}", i))
        }

        for i in 1..num_servers + 1 {
            server_addresses.push(format!("server-{}", i))
        }

        for addr in client_addresses.clone() {
            clients.insert(
                addr.clone(),
                Client {
                    addr: addr.clone(),
                    claimed_ids: vec![],
                    servers: server_addresses.clone(),
                    highest_id_seen: 0,
                    responses: HashMap::new(),
                },
            );
        }

        for addr in server_addresses {
            servers.insert(
                addr.clone(),
                Server {
                    addr: addr.clone(),
                    highest_id_seen: 0,
                },
            );
        }

        let rng: rand::rngs::StdRng = SeedableRng::seed_from_u64(network_seed);

        let mut sim = Simulator {
            in_flight,
            clients,
            servers,
            goal_per_client: num_ids_per_client,
            rng,
            time: 0,
        };

        for _ in 0..num_ids_per_client {
            for addr in client_addresses.clone() {
                sim.in_flight.push(Envelope {
                    from: "simulator".to_string(),
                    to: addr,
                    msg: Msg::StartRequest,
                    time: 1 + sim.rng.gen_range(1, 10),
                });
            }
        }

        sim.sort_in_flight();

        return sim;
    }

    pub fn run(&mut self) -> Result<(), String> {
        loop {
            if (self.goal_per_client * self.servers.len() * self.clients.len() * 100) < self.time {
                return Err("too many iterations".to_string());
            }

            if self.in_flight.len() == 0 {
                break;
            }

            self.sort_in_flight();

            let elem = self.in_flight.remove(0);

            self.process_item(elem);
        }

        Ok(())
    }

    fn sort_in_flight(&mut self) {
        self.in_flight
            .sort_by(|a: &Envelope, b: &Envelope| a.time.cmp(&b.time));
    }

    fn process_item(&mut self, e: Envelope) {
        println!("{:?}", e);
        let (from, replies) = match e.msg {
            Msg::StartRequest => match self.clients.get_mut(&e.to) {
                Some(client) => (client.get_addr().clone(), client.receive(e.msg, e.from)),
                None => panic!(),
            },
            Msg::Request(_) => match self.servers.get_mut(&e.to) {
                Some(server) => (server.get_addr().clone(), server.receive(e.msg, e.from)),
                None => panic!(),
            },
            Msg::Yes(_) => match self.clients.get_mut(&e.to) {
                Some(client) => (client.get_addr().clone(), client.receive(e.msg, e.from)),
                None => panic!(),
            },
            Msg::No(_) => match self.clients.get_mut(&e.to) {
                Some(client) => (client.get_addr().clone(), client.receive(e.msg, e.from)),
                None => panic!(),
            },
        };

        // We did work, thus moving the clock forward.
        self.time = e.time + 1;

        for (msg, to) in replies {
            self.in_flight.push(Envelope {
                from: from.clone(),
                to: to,
                msg: msg,
                time: self.time + self.rng.gen_range(1, 10),
            });
        }
    }

    pub fn validate_run(&self) -> Result<bool, String> {
        let clients: Vec<&Client> = self.clients.iter().map(|(_, v)| v).collect();

        // Make sure no two clients claimed the same Id.
        for i in 0..clients.len() {
            let a = clients[i];

            for j in (i + 1)..self.clients.len() {
                let b = clients[j];

                for a_id in a.claimed_ids.iter() {
                    for b_id in b.claimed_ids.iter() {
                        if a_id == b_id {
                            return Err(format!(
                                "both client {} and {} claimed id {}",
                                a.addr, b.addr, a_id
                            ));
                        }
                    }
                }
            }
        }

        // Make sure all clients claimed the amount of Ids they planned to.
        for c in clients {
            if c.claimed_ids.len() != self.goal_per_client {
                return Err(format!(
                    "expected {} to claim {} ids but got {}",
                    c.addr,
                    self.goal_per_client,
                    c.claimed_ids.len(),
                ));
            }
        }

        return Ok(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulator_increases_timestamp() {
        let mut sim = Simulator::new(2, 3, 1, 2);

        match sim.run() {
            Ok(_) => {}
            Err(e) => panic!(e),
        };

        assert_eq!(sim.time, 36);
    }
}
