use std::collections::HashMap;

pub type ID = u64;
pub type Addr = String;
pub type Timestamp = usize;

pub trait Receiver {
    fn receive(&mut self, m: Msg, addr: Addr) -> Vec<(Msg, Addr)>;
}

#[derive(Clone, Debug)]
pub enum Msg {
    StartRequest,
    Request(ID),
    Yes(ID),
    No(ID),
}

#[derive(Clone, Debug)]
pub struct Envelope {
    pub from: Addr,
    pub to: Addr,
    pub msg: Msg,
    pub time: Timestamp,
}

#[derive(Clone, Debug)]
pub struct Server {
    pub addr: Addr,
    pub highest_id_seen: ID,
}

impl Server {
    pub fn get_addr(&self) -> Addr {
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
pub struct Client {
    pub addr: Addr,
    pub servers: Vec<Addr>,
    // Track all yeses and nos for a given ID.
    pub responses: HashMap<ID, (usize, usize)>,
    pub highest_id_seen: ID,
    pub claimed_ids: Vec<ID>,
}

impl Client {
    pub fn get_addr(&self) -> Addr {
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
                Some((y, n)) => (y.clone(), n.clone()),
                None => (0, 0),
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
                if yes == self.servers.len() / 2 {
                    self.claimed_ids.push(id.clone());
                }

                return vec![];
            }
            Msg::No(id) => {
                let (yes, no) = get_responses_with_default(id);

                self.responses.insert(id, (yes, no + 1));

                // '==' not '>=' to prevent double retries.
                println!("no: {} {} {}", no, self.servers.len(), self.servers.len() /2 );
                if no == self.servers.len() / 2  {
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


