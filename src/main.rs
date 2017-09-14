use clap::{Arg, App};

extern crate clap;
extern crate rust_https_server;

use rust_https_server::start_server;

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
        .arg(Arg::with_name("cert")
            .long("cert").value_name("CERT_FILE")
            .takes_value(true))
        .arg(Arg::with_name("pk")
            .long("pk").value_name("PRIVATE_KEY_FILE")
            .takes_value(true))
        .arg(Arg::with_name("root_directory")
            .required(true)
            .value_name("ROOT_DIRECTORY"))
        .get_matches();

    let cert_matches = (matches.value_of("cert"), matches.value_of("pk"));
    let address = matches.value_of("address").unwrap();
    let port = matches.value_of("port").unwrap();
    let root_directory = matches.value_of("root_directory");

    let cert_info = match cert_matches {
        (Some(cert_path),Some(private_key_path)) => Some((cert_path, private_key_path)),
        (Some(_), None) => panic!("If the cert is specified, so must the private key"),
        (None, Some(_)) => panic!("If the private key is specified, so must the cert"),
        (None, None) => None
    };
    match root_directory {
        Some(root_dir) => {
            match start_server(root_dir, address, port, cert_info) {
                Ok(handle) => {
                    println!("listening on {}:{}", handle.ip, handle.port);
                    handle.handle.join().expect("Join failed");
                },
                Err(e) => eprintln!("{}", e)
            }
        },
        None => eprintln!("The root directory is required")
    }
}