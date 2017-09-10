use std::result::Result;
use std::collections::HashMap;
use std::io::{Write, BufRead};
use std::fmt;
use chrono::{FixedOffset,DateTime};

#[derive(Debug)]
pub struct Headers {
    headers: HashMap<String, String>
}

impl Headers {
    pub fn accept_encoding_gzip(&self) -> bool {
        match self.headers.get("Accept-Encoding") {
            Some(encoding) => encoding.contains("gzip"),
            None => false
        }
    }

    pub fn if_modified_since(&self) -> Option<DateTime<FixedOffset>> {
        match self.headers.get("If-Modified-Since") {
            Some(date_str) => parse_if_mod_by(date_str),
            None => None
        }
    }

    pub fn connection_keep_alive(&self) -> bool {
        match self.headers.get("Connection") {
            Some(value) => value.contains("keep-alive"),
            None => true
        }
    }
}

pub enum Request {
    EndRequests(),
    Get(String, Headers),
    Head(String, Headers),
    Options(Option<String>, Headers)
}

pub struct BadRequest {
    pub code: &'static str,
    pub reason: &'static str
}

pub fn parse_request<S: BufRead + Write>(buffed: &mut S) -> Result<Request, BadRequest> {
    let mut line_buff = String::new();
    let request_line = match buffed.read_line(&mut line_buff) {
        Ok(0) => return Ok(Request::EndRequests()),
        Ok(_) => parse_request_line(&line_buff),
        Err(e) => {
            eprintln!("Bad request line:{}", e);
            return Err(BadRequest {
                code: "400",
                reason: "Request line not understood",
            })
        }
    };

    match request_line {
        Ok((ref method, ref url)) if "GET".eq(method) => {
            let headers = parse_headers(buffed);
            return Ok(Request::Get(url.clone(), Headers { headers }));
        }
        Ok((ref method, ref url)) if "HEAD".eq(method) => {
            let headers = parse_headers(buffed);
            return Ok(Request::Head(url.clone(), Headers { headers }));
        }
        Ok((ref method, ref url)) if "OPTIONS".eq(method) && "*".eq(url) => {
            let headers = parse_headers(buffed);
            return Ok(Request::Options(None, Headers { headers }));
        }
        Ok((ref method, ref url)) if "OPTIONS".eq(method) => {
            let headers = parse_headers(buffed);
            return Ok(Request::Options(Some(url.clone()), Headers { headers }));
        }
        Ok((_, _)) => {
            Err(BadRequest { code: "405", reason: "Method not supported"})
        }
        Err(bad_request) => Err(bad_request)
    }
}

fn parse_request_line(input: &String) -> Result<(String, String), BadRequest> {
    let mut parts = input.split(' ');
    return match (parts.next(), parts.next()) {
        (Some(method), Some(url)) => Ok((method.to_string(), url.to_string())),
        (Some(method), None) => {
            eprintln!("No URL:method={}", method);
            Err(BadRequest { code: "400", reason: "" })
        },
        _ => Err(BadRequest { code: "400", reason: "" })
    }
}

fn parse_headers(reader: &mut BufRead) -> HashMap<String, String> {
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

fn parse_if_mod_by(date_str: &str) -> Option<DateTime<FixedOffset>> {
    match DateTime::parse_from_rfc2822(date_str) {
        Ok(dt) => Some(dt),
        Err(e) => {
            eprintln!("Error parsing {}:{}", date_str, e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_request_line,parse_headers};
    use std::io::BufReader;

    #[test]
    fn it_works() {
        let output = parse_request_line(&"GET / HTTP/1.1\r\n".to_string());
        match output {
            Ok((method, url)) => { assert_eq!(method, "GET"); assert_eq!(url, "/")},
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