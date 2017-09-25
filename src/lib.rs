use std::thread::{spawn, JoinHandle};
use std::net::{TcpListener};
use std::net::IpAddr;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use openssl::ssl::{SslMethod, SslAcceptorBuilder, SslAcceptor};
use openssl::pkey::PKey;
use openssl::x509::X509;
use openssl::stack::Stack;

extern crate bufstream;
extern crate chrono;
extern crate openssl;
extern crate byteorder;

mod server;

pub struct ServerHandle {
    pub ip: String, pub port: u16, pub handle: JoinHandle<()>
}

pub fn start_server(root_dir: &str, address: &str, port: &str, cert_info: Option<(&str, &str)>) -> Result<ServerHandle, String>{
    let root_path = Path::new(&root_dir);
    let acceptor = match cert_info {
        Some((cert_path, private_key_path)) => Some(create_acceptor(&cert_path, &private_key_path)),
        None => None
    };
    if !root_path.exists() {
        return Err(format!("Root path does not exist: {}", root_path.display()));
    } else {
        println!("binding to:{}:{}", address, port);
        match TcpListener::bind(format!("{}:{}", address, port)) {
            Ok(listener) => {
                let (ip, bind_port) = match listener.local_addr() {
                    Ok(addr) => {
                        let ip_string = match addr.ip() {
                            IpAddr::V4(v4) => {
                                let octets = v4.octets();
                                format!("{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3])
                            },
                            IpAddr::V6(v6) => {
                                let octets = v6.octets();
                                format!("{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
                                        octets[ 0], octets[ 1], octets[ 2], octets[ 3],
                                        octets[ 4], octets[ 5], octets[ 6], octets[ 7],
                                        octets[ 8], octets[ 9], octets[10], octets[11],
                                        octets[12], octets[13], octets[14], octets[15])
                            }
                        };
                        (ip_string, addr.port())
                    },
                    Err(_) => ("unknown".to_string(), 0)
                };

                let local_root_path = root_path.to_path_buf();
                let handle = spawn(move || {
                    match acceptor {
                        None => server::serve(listener, local_root_path.as_ref()),
                        Some(ac) => server::serve_https(listener, local_root_path.as_ref(), ac)
                    }
                });
                Ok(ServerHandle {ip, port: bind_port, handle})
            },
            Err(e) => Err(format!("Error on bind:{}", e))
        }
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