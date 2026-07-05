use std::thread;

mod server;
mod component;
mod renderer;

fn main() {
    println!("Hello, world!");

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
