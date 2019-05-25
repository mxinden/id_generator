extern crate queues;

use queues::*;
use std::collections::HashMap;

type ID = u64;
type Addr = String;
type Timestamp = u64;

trait Receiver {
    fn receive(&mut self, m: Msg, addr: Addr) -> Vec<(Msg, Addr)>;
}

#[derive(Clone, Debug)]
enum Msg {
    StartRequest,
    Request(ID),
    Yes(ID),
    No(ID),
}

#[derive(Clone, Debug)]
struct Envelope {
    from: Addr,
    to: Addr,
    msg: Msg,
    time: Timestamp,
}

#[derive(Clone, Debug)]
struct Server {
    addr: Addr,
    highest_id_seen: ID,
}

impl Server {
    fn get_addr(&self) -> Addr {
        self.addr.clone()
    }
}

impl Receiver for Server {
    fn receive(&mut self, m: Msg, from: Addr) -> Vec<(Msg, Addr)> {
        match m {
            Msg::StartRequest => {
                panic!();
            }
            Msg::Request(id) => {
                if self.highest_id_seen < id {
                    self.highest_id_seen = id;
                    return vec![(Msg::Yes(id), from)];
                }
                return vec![(Msg::No(id), from)];
            }
            Msg::Yes(_) => {
                panic!();
            }
            Msg::No(_) => {
                panic!();
            }
        }
    }
}

#[derive(Clone, Debug)]
struct Client {
    addr: Addr,
    servers: Vec<Addr>,
    // Track all yeses and nos for a given ID.
    responses: HashMap<ID, (usize, usize)>,
    highest_id_seen: ID,
    claimed_ids: Vec<ID>,
}

impl Client {
    fn get_addr(&self) -> Addr {
        self.addr.clone()
    }
}

impl Receiver for Client {
    fn receive(&mut self, m: Msg, _addr: Addr) -> Vec<(Msg, Addr)> {
        let to_all_servers = |servers: &Vec<Addr>, msg: Msg| -> Vec<(Msg, Addr)> {
            servers
                .iter()
                .map(|server| (msg.clone(), server.clone()))
                .collect()
        };

        let get_responses_with_default = |id: ID| -> (usize, usize) {
            match self.responses.get(&id) {
                Some((y, n)) => (y.clone(),n.clone()),
                None => (0,0),
            }
        };


        match m {
            Msg::StartRequest => {
                return to_all_servers(&self.servers, Msg::Request(self.highest_id_seen));
            }
            Msg::Yes(id) => {
                let (yes, no) = get_responses_with_default(id);

                self.responses.insert(id, (yes + 1, no));

                // '==' not '>=' to prevent double adding.
                if yes == self.servers.len() / 2 + 1 {
                    self.claimed_ids.push(id.clone());
                }

                return vec![];
            }
            Msg::No(id) => {
                let (yes, no) = get_responses_with_default(id);

                self.responses.insert(id, (yes, no + 1));

                // '==' not '>=' to prevent double retries.
                if no == self.servers.len() / 2 + 1 {
                    let next_id = self.highest_id_seen + 1;
                    self.highest_id_seen = next_id;
                    return to_all_servers(&self.servers, Msg::Request(next_id));
                }

                return vec![];
            }
            Msg::Request(_) => panic!(),
        }
    }
}

struct Simulator {
    in_flight: Queue<Envelope>,
    clients: HashMap<Addr, Client>,
    servers: HashMap<Addr, Server>,
    goal_per_client: usize,
}

impl Simulator {
    fn run(&mut self) -> String {
        while true {
            match self.in_flight.remove() {
                Err(e) => return e.into(),
                Ok(e) => self.process_item(e),
            }
        }

        "".to_string()
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
            // TODO: Handle result.
            self.in_flight.add(Envelope {
                from: from.clone(),
                to: to,
                msg: msg,
                // TODO: Change this timestamp.
                time: 2,
            });
        }
    }

    fn validate_run(&self) -> Result<bool, String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_run() {
        let mut in_flight: Queue<Envelope> = queue![];
        in_flight.add(Envelope {
            from: "simulator".to_string(),
            to: "client1".to_string(),
            msg: Msg::StartRequest,
            time: 1,
        });
        in_flight.add(Envelope {
            from: "simulator".to_string(),
            to: "client2".to_string(),
            msg: Msg::StartRequest,
            time: 2,
        });

        let server1 = Server {
            addr: "server1".to_string(),
            highest_id_seen: 0,
        };
        let server2 = Server {
            addr: "server2".to_string(),
            highest_id_seen: 0,
        };
        let server3 = Server {
            addr: "server3".to_string(),
            highest_id_seen: 0,
        };

        let client1 = Client {
            addr: "client1".to_string(),
            claimed_ids: vec![],
            servers: vec![
                "server1".to_string(),
                "server2".to_string(),
                "server3".to_string(),
            ],
            highest_id_seen: 0,
            responses: HashMap::new(),
        };
        let client2 = Client {
            addr: "client2".to_string(),
            claimed_ids: vec![],
            servers: vec![
                "server1".to_string(),
                "server2".to_string(),
                "server3".to_string(),
            ],
            highest_id_seen: 0,
            responses: HashMap::new(),
        };

        let mut clients = HashMap::new();
        clients.insert(client1.addr.clone(), client1);
        clients.insert(client2.addr.clone(), client2);

        let mut servers = HashMap::new();
        servers.insert(server1.addr.clone(), server1);
        servers.insert(server2.addr.clone(), server2);
        servers.insert(server3.addr.clone(), server3);

        let mut simulator = Simulator {
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
