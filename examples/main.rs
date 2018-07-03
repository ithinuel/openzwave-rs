extern crate openzwave_rs;

use openzwave_rs::*;
use std::collections::BTreeMap;
use std::io::stdin;
use std::sync::{Arc, Condvar, Mutex};

fn notify(pair: &Arc<(Mutex<bool>, Condvar)>) {
    let &(ref lock, ref cvar) = &**pair;
    let mut started = lock.lock().unwrap();
    *started = true;
    cvar.notify_one();
}

fn wait_and_rearm(pair: &Arc<(Mutex<bool>, Condvar)>) {
    // wait for the thread to start up
    let &(ref lock, ref cvar) = &**pair;
    let mut started = lock.lock().unwrap();
    while !*started {
        started = cvar.wait(started).unwrap();
    }
    *started = false;
}

#[derive(Debug)]
struct Node {
    id: u8,
    values: Vec<ValueID>,
}

fn main() {
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let notif_pair = pair.clone();

    let database = Arc::new(Mutex::new(BTreeMap::new()));
    let notif_db = database.clone();
    println!("enter any key to switch the Smart Plug.");
    println!("Press enter to quit");

    init("/dev/ttyAMA0", move |n| match n.get_type() {
        NotificationType::DriverReady => notify(&notif_pair),
        NotificationType::Notification | NotificationType::ControllerCommand => {
            // println!("{}: {:?}: {:?}", n.node_id(), t, n.code().unwrap());
        }
        NotificationType::NodeNew => {
            let id = n.node_id();
            println!("{}: Creating Node", id);
            let mut map = notif_db.lock().unwrap();
            map.insert(
                id,
                Node {
                    id: id,
                    values: Vec::new(),
                },
            );
        }
        NotificationType::NodeAdded => {
            println!("{}: Node added", n.node_id());
        }
        NotificationType::ValueAdded => {
            let value_id = n.value_id();
            let mut map = notif_db.lock().unwrap();
            map.get_mut(&n.node_id()).unwrap().values.push(value_id);
        }
        NotificationType::ValueChanged => {
            /*println!(
                "{}: Value changed: {}",
                n.node_id(),
                n.value_id().label()
            );*/
        }
        NotificationType::NodeQueriesComplete => {
            if n.node_id() == 3 {
                notify(&notif_pair);
            }
        }
        _ => {
            // println!("{}: {:?}: Ignoring", n.node_id(), t);
        }
    });

    // wait driver ready
    wait_and_rearm(&pair);

    // wait node query to complete on node 3
    wait_and_rearm(&pair);

    println!("Smart plug ready !");

    println!("{:#?}", *database.lock().unwrap());
    let switch = ValueID {
        home_id: 4100030940,
        node_id: 3,
        genre: ValueGenre::User,
        command_class_id: 37,
        instance: 1,
        value_index: 0,
        value_type: ValueType::Bool,
    };
    let power = ValueID {
        home_id: 4100030940,
        node_id: 3,
        genre: ValueGenre::User,
        command_class_id: 50,
        instance: 1,
        value_index: 8,
        value_type: ValueType::Decimal,
    };

    let mut state = false;

    let input = stdin();
    let mut cmd = String::new();
    let mut do_loop = true;
    while do_loop {
        cmd.clear();
        input.read_line(&mut cmd).expect("failed to read input");

        match cmd.trim() {
            "switch" => {
                switch.set_bool(state);
                state = !state;
            }
            "quit" => {
                do_loop = false;
            }
            _ => {
                println!("{}", power.get_string());
            }
        }
    }
}
