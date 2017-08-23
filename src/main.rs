use std::net::{TcpListener};
use std::path::Path;
use std::env;
use std::iter::{Iterator};
use std::string::String;
use std::slice::Iter;
use std::cmp::PartialEq;

extern crate bufstream;

mod server;

fn main() {
    println!("start");
    let args: Vec<String> = env::args().collect();
    let options = parse_ops(args.iter());

    let root_path = Path::new(&options.root);

    if !root_path.exists() {
        eprintln!("Root path does not exist: {}", root_path.display());
    } else {
        println!("binding to:{}:{}", options.address, options.port);
        println!("root={}->{}", options.root, root_path.display());
        match TcpListener::bind(format!("{}:{}", options.address, options.port)) {
            Ok(listener) => server::serve(listener, root_path),
            Err(e) => println!("Error on bind:{}", e)
        }
    }
}

enum ParseState {
    Normal(Options),
    AwaitingAddress(Options),
    AwaitingPort(Options),
}

fn parse_ops(args: Iter<String>) -> Options {
    let default_options = Options::default();

    let output = args.skip(1).fold(ParseState::Normal(default_options), |accum, ref arg| {
        match accum {
            ParseState::Normal(options) => {
                match arg.as_str() {
                    "-p" => ParseState::AwaitingPort(options),
                    "-a" => ParseState::AwaitingAddress(options),
                    path => ParseState::Normal(options.with_root(path.to_string()))
                }
            },
            ParseState::AwaitingAddress(options) => ParseState::Normal(options.with_address(arg.to_string())),
            ParseState::AwaitingPort(options) => ParseState::Normal(options.with_port(arg.to_string()))
        }
    } );

    match output {
        ParseState::Normal(options) => options,
        ParseState::AwaitingAddress(_) => panic!("Invalid options"),
        ParseState::AwaitingPort(_) => panic!("Invalid options")
    }
}

#[derive(Debug)]
struct Options {
    root: String,
    address: String,
    port: String
}

impl Options {
    pub fn default() -> Options {
        Options {
            root: ".".to_string(),
            address: "127.0.0.1".to_string(),
            port: "8080".to_string(),
        }
    }

    pub fn with_root(&self, root: String) -> Options {
        Options {
            root: root,
            address: self.address.clone(),
            port: self.port.clone(),
        }
    }

    pub fn with_address(&self, address: String) -> Options {
        Options {
            root: self.root.clone(),
            address: address,
            port: self.port.clone(),
        }
    }

    pub fn with_port(&self, port: String) -> Options {
        Options {
            root: self.root.clone(),
            address: self.address.clone(),
            port: port,
        }
    }
}

impl PartialEq for Options {
    fn eq(&self, other: &Options) -> bool {
        return self.port.eq(&other.port) &&
            self.address.eq(&other.address) &&
            self.root.eq(&other.root);
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_ops, Options};

    #[test]
    fn parse_ops_handles_empty() {
        let input = vec!["exec".to_string()];
        let output = parse_ops(input.iter());

        assert_eq!(Options::default(), output);
    }

    #[test]
    fn parse_ops_handles_just_dir() {
        let input = vec!["exec".to_string(), "dummy_dir".to_string()];
        let output = parse_ops(input.iter());

        assert_eq!(Options::default().with_root("dummy_dir".to_string()), output);
    }

    #[test]
    fn parse_ops_handles_just_port() {
        let input = vec!["exec".to_string(), "-p".to_string(), "1234".to_string()];
        let output = parse_ops(input.iter());

        assert_eq!(Options::default().with_port("1234".to_string()), output);
    }

    #[test]
    fn parse_ops_handles_just_address() {
        let input = vec!["exec".to_string(), "-a".to_string(), "1.2.3.4".to_string()];
        let output = parse_ops(input.iter());

        assert_eq!(Options::default().with_address("1.2.3.4".to_string()), output);
    }

    #[test]
    fn parse_ops_handles_all() {
        let input = vec!["exec".to_string(), "-a".to_string(), "1.2.3.4".to_string(), "-p".to_string(), "1234".to_string(), "dummy_dir".to_string()];
        let output = parse_ops(input.iter());
        let expected = Options {
            root: "dummy_dir".to_string(),
            address: "1.2.3.4".to_string(),
            port: "1234".to_string()
        };

        assert_eq!(expected, output);
    }
}