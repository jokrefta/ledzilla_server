use std::thread;

use log::{trace, error};

mod component;
mod renderer;
mod server;


pub fn run_server() {
    let (s, r) = std::sync::mpsc::channel::<String>();

    let receiver_thread = thread::spawn(move || {
        loop {
            let val = r.recv().unwrap();
            println!("{}", val);
            println!("");
        }
    });

    rouille::start_server("127.0.0.1:8080", move |req| {
        server::handle_request(s.clone(), req)
    });

}
