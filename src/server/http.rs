use std::result::Result;
use std::collections::HashMap;
use std::io::{BufRead};

pub enum Request {
    GET {
        url: String
    }
}

pub struct BadRequest {
    pub code: &'static str,
    pub reason: &'static str
}

pub fn parse_request_line(input: &String) -> Result<Request, BadRequest> {
    let mut parts = input.split(' ');
    match (parts.next(), parts.next()) {
        (Some("GET"), Some(url)) => {
            Ok(Request::GET { url: url.to_string() })
        },
        _ => Err(BadRequest { code: "400", reason: "" })
    }
}

pub fn parse_headers(reader: &mut BufRead) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) if line.len() < 3 => break,
            Ok(_) => {
                match line.find(':') {
                    Some(index) => {
                        let (name, value) = line.split_at(index);
                        let mut value_string = value.to_string();
                        value_string.remove(0);

                        headers.insert(name.to_string(), value_string.trim().to_string());
                    },
                    None => {}
                }
            },
            Err(e) => print!("Error reading header line:{}", e)
        }
    }
    return headers;
}

#[cfg(test)]
mod tests {
    use super::{parse_request_line,parse_headers};
    use super::Request;
    use std::io::BufReader;

    #[test]
    fn it_works() {
        let output = parse_request_line(&"GET / HTTP/1.1\r\n".to_string());
        match output {
            Ok(Request::GET { url }) => { assert_eq!(url, "/")},
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_headers_handles_empty_header() {
        let mut input = BufReader::new("\r\n\r\n".as_bytes());
        let output = parse_headers(&mut input);

        assert_eq!(0, output.len());
    }

    #[test]
    fn parse_headers_handles_single_header() {
        let mut input = BufReader::new("Dummy: test\r\n\r\n".as_bytes());
        let output = parse_headers(&mut input);

        assert_eq!(1, output.len());
        assert_eq!(output.get(&"Dummy".to_string()), Some(&"test".to_string()));
    }

    #[test]
    fn parse_headers_handles_2_headers() {
        let mut input = BufReader::new("Dummy: test\r\nDummy2: test2\r\n\r\n".as_bytes());
        let output = parse_headers(&mut input);

        assert_eq!(2, output.len());
        assert_eq!(output.get(&"Dummy".to_string()), Some(&"test".to_string()));
        assert_eq!(output.get(&"Dummy2".to_string()), Some(&"test2".to_string()));
    }
}