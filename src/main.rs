use std::net::{TcpListener};
use std::path::Path;
use clap::{Arg, App};

extern crate clap;
extern crate bufstream;

mod server;

fn main() {
    println!("start");
    let matches = App::new("http-server")
        .version("1.0")
        .about("A simple web server")
        .arg(Arg::with_name("address")
            .short("a")
            .value_name("BIND_ADDRESS")
            .default_value("127.0.0.1")
            .takes_value(true))
        .arg(Arg::with_name("port")
            .short("p")
            .value_name("PORT_NUMBER")
            .default_value("8080")
            .takes_value(true))
        .arg(Arg::with_name("root_directory")
            .required(true)
            .value_name("ROOT_DIRECTORY"))
        .get_matches();


    let root_path = Path::new(matches.value_of("root_directory").unwrap());

    if !root_path.exists() {
        eprintln!("Root path does not exist: {}", root_path.display());
    } else {
        println!("binding to:{}:{}", matches.value_of("address").unwrap(), matches.value_of("port").unwrap());
        println!("root={}->{}", matches.value_of("root_directory").unwrap(), root_path.display());
        match TcpListener::bind(format!("{}:{}", matches.value_of("address").unwrap(), matches.value_of("port").unwrap())) {
            Ok(listener) => server::serve(listener, root_path),
            Err(e) => println!("Error on bind:{}", e)
        }
    }
}