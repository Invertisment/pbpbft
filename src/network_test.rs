
#[cfg(test)]
mod network_basic_test {

    use crate::dto::{ID,Commit};
    use crate::node::{Message};
    use crate::network::{Network};

    fn send_requests(net: &mut Network, count: usize) {
        for i in 0..count {
        net.queue_add(
            Message::commit(
                100,
                i as ID,
                Commit::new(1, 1, String::from(format!("digest {}", i)), i as ID, i as ID)));
        }
    }

    fn mk_net() -> Network {
        Network::new(5, 10)
    }

    #[test]
    fn network_empty() {
        let mut net: Network = Network::new(0, 10);
        send_requests(&mut net, 5);
        match net.tick() {
            Ok(_res) => panic!("Should fail with no nodes"),
            Err(e) => assert_eq!(e, "No nodes were found"),
        };
    }

    #[test]
    fn network_no_reqs() {
        let mut net: Network = mk_net();
        match net.tick() {
            Ok(_res) => panic!("Should fail with no requests"),
            Err(e) => assert_eq!(e, "No more requests"),
        };
    }

    #[test]
    fn network_drops_packets() {
        let mut net: Network = mk_net();
        send_requests(&mut net, 5);
        match net.tick() {
            Ok(res) => assert!(res),
            Err(e) => panic!(e),
        };
    }

    #[test]
    fn network_count_packets() {
        let mut net: Network = mk_net();
        send_requests(&mut net, 5);
        let mut res : i64 = 0;
        loop {
            match net.tick() {
                Ok(b) => if b {
                    res = &res + 1
                },
                Err(_e) => break,
            };
        }
        assert_eq!(res, 5)
    }

    #[test]
    fn node_remove() {
        let mut net = Network::new(2, 5);
        match net.remove_node(1).unwrap().join() {
            Ok(_) => {},
            Err(e) => panic!(e),
        };
        send_requests(&mut net, 5);
        match net.tick() {
            Ok(res) => assert!(res),
            Err(e) => panic!(e),
        };
        match net.tick() {
            Ok(res) => assert!(!res),
            Err(e) => panic!(e),
        };
    }

}

#[cfg(test)]
mod network_interaction_test {
    use crate::dto::{ID,PrePrepare};
    use crate::node::{Message,NodeCtrl,State};
    use crate::network::{Network};
    use std::sync::{Mutex,Arc,RwLock};
    use std::collections::HashMap;

    fn get_preprepare_size(maybe_node: Option<&NodeCtrl>) -> Result<usize, String> {
        if maybe_node.is_none() {
            return Err("Node not found".to_owned());
        }
        let node = maybe_node.unwrap();
        let state_mutex: Arc<Mutex<State>> = node.get_state();
        let state_lock = state_mutex.lock();
        if state_lock.is_err() {
            return Err("Node not found".to_owned());
        }
        let state: std::sync::MutexGuard<'_, State, > = state_lock.unwrap();
        let preprepares: &HashMap<ID, Arc<RwLock<PrePrepare>>> = state.get_preprepares();
        Ok(preprepares.len())
    }

    #[test]
    fn preprepare_should_reach_node() {
        let mut net = Network::new(2, 5);
        let sender = 1337 as ID;
        let target = 1 as ID;
        net.queue_add(Message::preprepare(
            sender,
            target,
            PrePrepare::new(0, 0, "digest".to_owned(), sender, "message".to_owned())));
        match get_preprepare_size(net.get_node(&target)) {
            Ok(size) => assert_eq!(size, 0),
            Err(msg) => panic!(msg),
        }
        match net.tick() {
            Ok(res) => assert!(res),
            Err(e) => panic!(e),
        };
        match get_preprepare_size(net.get_node(&target)) {
            Ok(size) => assert_eq!(size, 1),
            Err(msg) => panic!(msg),
        }
    }

}
