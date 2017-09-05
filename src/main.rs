use std::net::{TcpListener};
use std::path::Path;
use clap::{Arg, App};
use std::fs::File;
use std::io::Read;
use openssl::ssl::{SslMethod, SslAcceptorBuilder, SslAcceptor};
use openssl::pkey::PKey;
use openssl::x509::X509;
use openssl::stack::Stack;

extern crate clap;
extern crate bufstream;
extern crate chrono;
extern crate openssl;

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



    match matches.value_of("root_directory") {
        Some(root_dir) => {
            let root_path = Path::new(root_dir);
            let acceptor = match (matches.value_of("cert"), matches.value_of("pk")) {
                (Some(cert_path),Some(private_key_path)) => Some(create_acceptor(&cert_path, &private_key_path)),
                (Some(_), None) => panic!("If the cert is specified, so must the private key"),
                (None, Some(_)) => panic!("If the private key is specified, so must the cert"),
                (None, None) => None
            };

            if !root_path.exists() {
                eprintln!("Root path does not exist: {}", root_path.display());
            } else {
                println!("binding to:{}:{}", matches.value_of("address").unwrap(), matches.value_of("port").unwrap());
                println!("root={}->{}", matches.value_of("root_directory").unwrap(), root_path.display());
                match TcpListener::bind(format!("{}:{}", matches.value_of("address").unwrap(), matches.value_of("port").unwrap())) {
                    Ok(listener) => match acceptor {
                        None => server::serve(listener, root_path),
                        Some(ac) => server::serve_https(listener, root_path, ac)
                    },
                    Err(e) => eprintln!("Error on bind:{}", e)
                }
            }
        },
        None => eprintln!("The root directory is required")
    }
}

fn create_acceptor(cert_path: &str, private_key_path: &str) -> SslAcceptor {
    let identity_result = load_cert(cert_path, private_key_path);

    match identity_result {
        Ok((pkey, cert)) => {
            let chain:Stack<X509> = Stack::new().unwrap();
            SslAcceptorBuilder::mozilla_intermediate(SslMethod::tls(), &pkey, &cert, &chain)
                .unwrap()
                .build()
        },
        Err(e) => panic!("Error loading cert:{}", e)
    }
}

fn load_cert(cert_path: &str, private_key_path: &str) -> Result<(PKey, X509), String> {
    let pkey = match File::open(private_key_path) {
        Ok(mut file) => {
            let mut pkey_buff: Vec<u8> = vec![];
            file.read_to_end(&mut pkey_buff).unwrap();
            match PKey::private_key_from_pem(pkey_buff.as_ref()) {
                Ok(pkey) => pkey,
                Err(e) => {
                    return Err(format!("Error extracting private key:{}", e));
                }
            }
        },
        Err(e) => {
            return Err(format!("Error reading private key:{}", e));
        }
    };
    let x509 = match File::open(cert_path) {
        Ok(mut file) => {
            let mut cert_buff: Vec<u8> = vec![];
            file.read_to_end(&mut cert_buff).unwrap();
            match X509::from_pem(cert_buff.as_ref()) {
                Ok(x509) => x509,
                Err(e) => {
                    return Err(format!("Error extracting cert:{}", e));
                }
            }
        },
        Err(e) => {
            return Err(format!("Error reading cert:{}", e));
        }
    };

    return Ok((pkey, x509));
}