use std::io::{Read, Write, BufRead};
use bufstream::BufStream;
use chrono::{FixedOffset,DateTime, Utc};
use super::content_manager::{ContentHandle, ContentManager};
use super::http::{parse_headers, parse_request_line, Request, BadRequest};

pub fn handle_client<H: ContentHandle, S: Read + Write>(stream: S, manager: &ContentManager<H>) {
    println!("in handle_client");

    let mut buffed = BufStream::new(stream);
    let mut line_buff = String::new();
    let request_line = match buffed.read_line(&mut line_buff) {
        Ok(_) => parse_request_line(&line_buff),
        Err(e) => {
            eprintln!("Bad request line:{}", e);
            Err(BadRequest {
                code: "400",
                reason: "Request line not understood",
            })
        }
    };

    let headers = parse_headers(&mut buffed);
    for (key, value) in &headers {
        println!("Header:{}->{}", key, value);
    }

    let gzip_encoding = match headers.get("Accept-Encoding") {
        Some(encoding) => encoding.contains("gzip"),
        None => false
    };

    let if_mod_since = match headers.get("If-Modified-Since") {
        Some(date_str) => parse_if_mod_by(date_str),
        None => None
    };

    match request_line {
        Ok(Request::GET { url }) => {
            handle_get(url, gzip_encoding, if_mod_since, false, &mut buffed, manager);
        }
        Ok(Request::HEAD { url }) => {
            handle_get(url, gzip_encoding, if_mod_since, true, &mut buffed, manager);
        }
        Ok(Request::OPTIONS { url }) => {
            handle_options(url, &mut buffed, manager);
        }
        Err(BadRequest { code, reason }) => {
            eprintln!("Error:{}/{}", code, reason);
            write!(&mut buffed, "HTTP/1.1 {} {}\n\n", code, reason).expect("Error while writing to output\n");
        }
    }

    println!("end handle_client")
}

fn handle_get<H: ContentHandle>(url: String, gzip_encoding: bool, if_mod_since: Option<DateTime<FixedOffset>>, suppress_entity: bool, buffed: &mut Write, manager: &ContentManager<H>) {
    match manager.find_content(&url, gzip_encoding) {
        Some(handle) => {
            match if_mod_since {
                Some(dt) => {
                    if handle.is_mod_since(&dt) {
                        write!(buffed, "HTTP/1.1 304 Not Modified\n").expect("Error while writing to output\n");
                        return;
                    }
                },
                None => {}
            }
            let content_length = format!("{}", handle.content_length());
            let last_modified = handle.mod_time().to_rfc2822();
            let mut headers: Vec<(&str, &str)> = vec![
                ("Content-Length", content_length.as_ref()),
                ("Last-Modified", last_modified.as_ref())
            ];
            if handle.is_gzipped() {
                headers.push(("Content-Encoding", "gzip"));
            }
            write_response(buffed, "200", "OK", headers, if suppress_entity {None} else { Some(handle) });
        }
        None => {
            write!(buffed, "HTTP/1.1 404 Not Found\n\n").expect("Error while writing to output\n");
        }
    }
}

fn handle_options<H: ContentHandle>(url: String, buffed: &mut Write,  manager: &ContentManager<H>) {
    if "*".eq(&url) {
        let headers: Vec<(&str, &str)> = vec![
            ("Allow", "OPTIONS, GET, HEAD"),
            ("Content-Length", "0")
        ];
        let handler: Option<H> = None;
        write_response(buffed, "200", "OK", headers, handler);
    } else {
        match manager.find_content(&url, false) {
            Some(_) => {
                let headers: Vec<(&str, &str)> = vec![
                    ("Allow", "OPTIONS, GET, HEAD"),
                    ("Content-Length", "0")
                ];
                let handler: Option<H> = None;
                write_response(buffed, "200", "OK", headers, handler);
            }
            None => {
                write!(buffed, "HTTP/1.1 404 Not Found\n\n").expect("Error while writing to output\n");
            }
        }
    }
}

fn write_response<H: ContentHandle>(buffed: &mut Write, code: &str, text: &str, headers: Vec<(&str, &str)>, handler: Option<H>) {
    write!(buffed, "HTTP/1.1 {} {}\n", code, text).expect("Error while writing to output\n");
    write!(buffed, "Connection: close\n").expect("Error while writing to output\n");
    write!(buffed, "Date: {}\n", Utc::now().to_rfc2822()).expect("Error while writing to output\n");
    write!(buffed, "Server: rust-http2-server\n").expect("Error while writing to output\n");
    for (name, value) in headers {
        writeln!(buffed, "{}: {}", name, value).expect("Error while writing header\n");
    }
    write!(buffed, "\n").expect("Error while terminating the headers\n");
    match handler {
        Some(mut h) => h.write_to(buffed),
        None => {}
    }
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
    #[test]
    fn read_header_works() {

    }
}