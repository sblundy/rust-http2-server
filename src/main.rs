use std::net::{TcpListener};
use std::path::Path;
use std::env;

extern crate bufstream;

mod server;

fn main() {
    println!("start");
    let args: Vec<String> = env::args().collect();

    let root = &args[1];
    let root_path = Path::new(root);
    if !root_path.exists() {
        eprintln!("Root path does not exist: {}", root_path.display());
    } else {
        println!("root={}->{}", root, root_path.display());
        match TcpListener::bind("127.0.0.1:8080") {
            Ok(listener) => server::serve(listener, root_path),
            Err(e) => println!("Error on bind:{}", e)
        }
    }
}