extern crate id_generator;
extern crate queues;

use id_generator::{Msg, Envelope, Client, Server, Addr, Receiver};
use std::collections::HashMap;
use queues::*;

pub struct Simulator {
    pub in_flight: Queue<Envelope>,
    pub clients: HashMap<Addr, Client>,
    pub servers: HashMap<Addr, Server>,
    pub goal_per_client: usize,
}

impl Simulator {
    pub fn run(&mut self) -> String {
        loop {
            match self.in_flight.remove() {
                Err(e) => return e.into(),
                Ok(e) => self.process_item(e),
            }
        }
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

        for (msg, to) in replies {
            match self.in_flight.add(Envelope {
                from: from.clone(),
                to: to,
                msg: msg,
                // TODO: Change this timestamp.
                time: 2,
            }) {
                Ok(_) => {}
                Err(e) => panic!(e.to_string()),
            }
        }
    }

    pub fn validate_run(&self) -> Result<bool, String> {
        let clients: Vec<&Client> = self.clients.iter().map(|(_, v)| v).collect();

        // Make sure no two clients claimed the same ID.
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

        // Make sure all clients claimed the amount of IDs they planned to.
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
