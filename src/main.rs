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
    fn getAddr(&self) -> Addr {
        self.addr.clone()
    }
}

impl Receiver for Server {
    fn receive(&mut self, m: Msg, from: Addr) -> Vec<(Msg, Addr)> {
        match m {
            Msg::Request(id) => {
                if self.highest_id_seen < id {
                    // TODO: Make sure we increment our ID here.
                    return vec![(Msg::Yes(id), from)];
                }
            }
            Msg::Yes(_) => {
                panic!();
            }
            Msg::No(_) => {
                panic!();
            }
        }

        panic!();
    }
}

#[derive(Clone, Debug)]
struct Client {
    addr: Addr,
    claimed_ids: Vec<ID>,
}

impl Receiver for Client {
    fn receive(&mut self, m: Msg, addr: Addr) -> Vec<(Msg, Addr)> {
        match m {
            Msg::Yes(id) => {
                // TODO: We need a quorum, not only one YES.
                self.claimed_ids.push(id);
            }
            Msg::No(id) => {}
            Msg::Request(_) => panic!(),
        }
        Vec::new()
    }
}

struct Simulator {
    in_flight: Queue<Envelope>,
    clients: HashMap<Addr, Client>,
    servers: HashMap<Addr, Server>,
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
        println!("process item");

        match e.msg {
            Msg::Request(id) => match self.servers.get_mut(&e.to) {
                Some(server) => {
                    let replies: Vec<(Msg, Addr)> = server.receive(e.msg, e.from);

                    println!("got {} replies", replies.len());

                    for (msg, to) in replies {
                        // TODO: Handle result.
                        self.in_flight.add(Envelope {
                            from: server.getAddr(),
                            to: to,
                            msg: msg,
                            // TODO: Change this timestamp.
                            time: 2,
                        });
                    }
                }
                None => panic!(),
            },
            Msg::Yes(_) => match self.clients.get_mut(&e.to) {
                Some(client) => {
                    client.receive(e.msg, e.from);
                }
                None => panic!(),
            },
            Msg::No(id) => match self.clients.get_mut(&e.to) {
                Some(client) => {
                    client.receive(e.msg, e.from);
                }
                None => panic!(),
            },
        }
    }

    fn validate_run(&self) -> Result<bool, String> {
        let clients: Vec<&Client> = self.clients.iter().map(|(k, v)| v).collect();

        // Make sure no two clients claim the same ID.
        for i in 0..clients.len() {
            let a = clients[i];

            for j in (i + 1)..self.clients.len() {
                let b = clients[j];

                for a_id in a.claimed_ids.iter() {
                    for b_id in b.claimed_ids.iter() {
                        if a_id == b_id {
                            return Err(format!(
                                "both client {} {} and {} {} claimed id {}",
                                i, a.addr, j, b.addr, a_id
                            ));
                        }
                    }
                }
            }
        }

        // TODO: Make sure all clients claimed the amount of IDs they planned to.
        return Ok(true);
    }
}

fn main() {
    let mut in_flight: Queue<Envelope> = queue![];
    in_flight.add(Envelope {
        from: "client1".to_string(),
        to: "server1".to_string(),
        msg: Msg::Request(1),
        time: 1,
    });
    in_flight.add(Envelope {
        from: "client2".to_string(),
        to: "server2".to_string(),
        msg: Msg::Request(1),
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
    };
    let client2 = Client {
        addr: "client2".to_string(),
        claimed_ids: vec![],
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
    };

    simulator.run();

    println!("in flight: {:?}", simulator.in_flight);

    for (k, v) in simulator.clients.iter() {
        println!("claimed ids: {:?}", v.claimed_ids)
    }

    println!("done");

    let validation = simulator.validate_run();

    match validation {
        Ok(_) => {}
        Err(e) => panic!(e),
    }
}
