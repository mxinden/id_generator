use std::collections::HashMap;

pub type Id = u64;
pub type Addr = String;
pub type Timestamp = usize;

pub trait Receiver {
    fn receive(&mut self, _: Msg, _: Addr) -> Vec<(Msg, Addr)>;
}

#[derive(Clone, Debug)]
pub enum Msg {
    StartRequest,
    Request(Id),
    Yes(Id),
    No(Id),
}

impl PartialEq for Msg {
    fn eq(&self, other: &Msg) -> bool {
        match self {
            Msg::StartRequest => match other {
                Msg::StartRequest => {
                    true
                }
                Msg::Request(_) => {
                    false
                }
                Msg::Yes(_) => {
                    false
                }
                Msg::No(_) => {
                    false
                }
            },
            Msg::Request(id) => match other {
                Msg::StartRequest => {
                    false
                }
                Msg::Request(other_id) => {
                    id == other_id
                }
                Msg::Yes(_) => {
                    false
                }
                Msg::No(_) => {
                    false
                }
            },
            Msg::Yes(yes) => match other {
                Msg::StartRequest => {
                    false
                }
                Msg::Request(_) => {
                    false
                }
                Msg::Yes(other_yes) => {
                    yes == other_yes
                }
                Msg::No(_) => {
                    false
                }
            },
            Msg::No(no) => match other {
                Msg::StartRequest => {
                    false
                }
                Msg::Request(_) => {
                    false
                }
                Msg::Yes(_) => {
                    false
                }
                Msg::No(other_no) => {
                    no == other_no
                }
            },
        }
    }
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
    pub highest_id_seen: Id,
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
                    vec![(Msg::Yes(id), from)]
                } else {
                    vec![(Msg::No(id), from)]
                }
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
    // Track all yeses and nos for a given Id.
    pub responses: HashMap<Id, (usize, usize)>,
    pub highest_id_seen: Id,
    pub claimed_ids: Vec<Id>,
}

impl Client {
    pub fn get_addr(&self) -> Addr {
        self.addr.clone()
    }
}

impl Receiver for Client {
    fn receive(&mut self, m: Msg, _addr: Addr) -> Vec<(Msg, Addr)> {
        fn to_all_servers(servers: &[Addr], msg: Msg) -> Vec<(Msg, Addr)> {
            servers
                .iter()
                .map(|server| (msg.clone(), server.clone()))
                .collect()
        };

        let mut get_responses_with_default =
            |id: Id| -> (usize, usize) { *self.responses.get_mut(&id).unwrap_or(&mut (0, 0)) };

        match m {
            Msg::StartRequest => {
                self.highest_id_seen += 1;
                to_all_servers(&self.servers, Msg::Request(self.highest_id_seen))
            }
            Msg::Yes(id) => {
                let (yes, no) = get_responses_with_default(id);

                let new_yes = yes + 1;
                self.responses.insert(id, (new_yes, no));

                // '==' not '>=' to prevent double adding.
                if new_yes == (self.servers.len() / 2) + 1 {
                    self.claimed_ids.push(id);
                }


                vec![]
            }
            Msg::No(id) => {
                let (yes, no) = get_responses_with_default(id);

                let new_no = no + 1;
                self.responses.insert(id, (yes, new_no));

                // When #servers == 1, retry at #no == 1
                // When #servers == 2, retry at #no == 1
                // When #servers == 3, retry at #no == 2
                // When #servers == 4, retry at #no == 2
                // ...
                if new_no == self.servers.len() - self.servers.len() / 2 {
                    self.highest_id_seen += 1;
                    return to_all_servers(&self.servers, Msg::Request(self.highest_id_seen));
                }

                vec![]
            }
            Msg::Request(_) => panic!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_receive_one_out_of_two_yes() {
        let mut c = Client {
            addr: "some".to_string(),
            claimed_ids: vec![],
            servers: vec!["server-1".to_string(), "server-2".to_string()],
            highest_id_seen: 1,
            responses: HashMap::new(),
        };

        assert_eq!(0, c.receive(Msg::Yes(1), "server-1".to_string()).len());

        // Not claimed an id yet.
        assert_eq!(0, c.claimed_ids.len());

        assert_eq!(0, c.receive(Msg::Yes(1), "server-2".to_string()).len());

        // Now claimed one id.
        assert_eq!(1, c.claimed_ids.len());
    }

    #[test]
    fn client_receive_no_msg_and_retry() {
        let mut c = Client {
            addr: "some".to_string(),
            claimed_ids: vec![],
            servers: vec!["server-1".to_string(), "server-2".to_string()],
            highest_id_seen: 1,
            responses: HashMap::new(),
        };

        let responses = c.receive(Msg::No(1), "server-2".to_string());

        assert_eq!(
            responses,
            vec![
                (Msg::Request(2), "server-1".to_string()),
                (Msg::Request(2), "server-2".to_string())
            ]
        );
    }

    #[test]
    fn client_with_three_servers() {
        let mut c = Client {
            addr: "some".to_string(),
            claimed_ids: vec![],
            servers: vec![
                "server-1".to_string(),
                "server-2".to_string(),
                "server-3".to_string(),
            ],
            highest_id_seen: 0,
            responses: HashMap::new(),
        };

        assert_eq!(
            c.receive(Msg::StartRequest, "simulator".to_string()),
            vec![
                (Msg::Request(1), "server-1".to_string()),
                (Msg::Request(1), "server-2".to_string()),
                (Msg::Request(1), "server-3".to_string()),
            ]
        );

        assert_eq!(
            c.receive(Msg::StartRequest, "simulator".to_string()),
            vec![
                (Msg::Request(2), "server-1".to_string()),
                (Msg::Request(2), "server-2".to_string()),
                (Msg::Request(2), "server-3".to_string()),
            ]
        );

        assert_eq!(
            c.receive(Msg::StartRequest, "simulator".to_string()),
            vec![
                (Msg::Request(3), "server-1".to_string()),
                (Msg::Request(3), "server-2".to_string()),
                (Msg::Request(3), "server-3".to_string()),
            ]
        );

        assert_eq!(c.receive(Msg::Yes(1), "simulator".to_string()), vec![]);

        // Expect client not to have claimed any ids so far.
        assert_eq!(c.claimed_ids, vec![]);
    }
}
