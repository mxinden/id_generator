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
        while self.in_flight.len() > 0 {
            if (self.goal_per_client * self.servers.len() * self.clients.len() * 100) < self.time {
                return Err("too many iterations".to_string());
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
        let (from, replies) = match e.msg {
            Msg::StartRequest | Msg::Yes(_) | Msg::No(_) => {
                let c = self.clients.get_mut(&e.to).unwrap();
                (c.get_addr().clone(), c.receive(e.msg, e.from))
            }
            Msg::Request(_) => {
                let s = self.servers.get_mut(&e.to).unwrap();
                (s.get_addr().clone(), s.receive(e.msg, e.from))
            }
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

    pub fn validate_run(&self) -> Result<(), String> {
        let clients: Vec<&Client> = self.clients.iter().map(|(_, v)| v).collect();

        Simulator::no_duplicate_ids(&clients)?;
        Simulator::clients_reached_their_goal(&clients, self.goal_per_client)?;

        return Ok(());
    }

    fn clients_reached_their_goal(
        clients: &Vec<&Client>,
        goal_per_client: usize,
    ) -> Result<(), String> {
        clients
            .iter()
            .map(|c| {
                if c.claimed_ids.len() != goal_per_client {
                    Err(format!(
                        "expected {} to claim {} ids but got {}",
                        c.addr,
                        goal_per_client,
                        c.claimed_ids.len(),
                    ))
                } else {
                    Ok(())
                }
            })
            .collect::<Result<Vec<()>, String>>()?;

        Ok(())
    }

    fn no_duplicate_ids(clients: &Vec<&Client>) -> Result<(), String> {
        for i in 0..clients.len() {
            let a = clients[i];

            for j in (i + 1)..clients.len() {
                let b = clients[j];

                let mut test_map = HashMap::new();

                a.claimed_ids
                    .iter()
                    .map(|id| test_map.insert(id, id))
                    // Make sure none are inserted twice.
                    .fold(Ok(()), |acc, v| {
                        acc.and_then(|()| match v {
                            Some(id) => {
                                Err(format!("expected {} not to claim id {} twice", a.addr, id))
                            }
                            None => Ok(()),
                        })
                    })?;

                b.claimed_ids
                    .iter()
                    .map(|id| test_map.insert(id, id))
                    .fold(Ok(()), |acc, v| {
                        acc.and_then(|()| match v {
                            Some(id) => {
                                Err(format!("both {} and {} claimed id {}", a.addr, b.addr, id))
                            }
                            None => Ok(()),
                        })
                    })?;
            }
        }

        Ok(())
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

        assert_eq!(sim.time, 41);
    }

    #[test]
    fn simulator_no_duplicate_ids() {
        let mut c_a = Client {
            addr: "client-a".to_string(),
            claimed_ids: vec![],
            servers: vec![],
            highest_id_seen: 0,
            responses: HashMap::new(),
        };
        c_a.claimed_ids = vec![1, 2, 3, 4, 20];

        let mut c_b = Client {
            addr: "client-b".to_string(),
            claimed_ids: vec![],
            servers: vec![],
            highest_id_seen: 0,
            responses: HashMap::new(),
        };
        c_b.claimed_ids = vec![5, 6, 7, 8, 20];

        assert_eq!(
            Simulator::no_duplicate_ids(&vec![&c_a, &c_b]),
            Err("both client-a and client-b claimed id 20".to_string()),
        );
    }

    #[test]
    fn simulator_clients_reach_their_goal() {
        let mut c_a = Client {
            addr: "client-a".to_string(),
            claimed_ids: vec![],
            servers: vec![],
            highest_id_seen: 0,
            responses: HashMap::new(),
        };
        c_a.claimed_ids = vec![1, 2, 3, 4];

        let mut c_b = Client {
            addr: "client-b".to_string(),
            claimed_ids: vec![],
            servers: vec![],
            highest_id_seen: 0,
            responses: HashMap::new(),
        };
        c_b.claimed_ids = vec![5, 6, 7];

        assert_eq!(
            Simulator::clients_reached_their_goal(&vec![&c_a, &c_b], 4),
            Err("expected client-b to claim 4 ids but got 3".to_string()),
        );
    }
}
