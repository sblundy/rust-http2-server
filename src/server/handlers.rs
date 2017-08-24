use std::io::{Read, Write, BufRead};
use bufstream::BufStream;
use super::content_manager::{ContentHandle, ContentManager};
use super::http::{parse_headers, parse_request_line, Request, BadRequest};

pub fn handle_client<H: ContentHandle, S: Read + Write>(stream: S, manager: &ContentManager<H>) {
    println!("in handle_client");

    let mut buffed = BufStream::new(stream);
    let mut line_buff = String::new();
    let request_line = match buffed.read_line(&mut line_buff) {
        Ok(_) => parse_request_line(&line_buff),
        Err(e) => {
            println!("Bad request line:{}", e);
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

    match request_line {
        Ok(Request::GET { url }) => {
            handle_get(url, gzip_encoding, &mut buffed, manager);
        }
        Err(BadRequest { code, reason }) => {
            println!("Error:{}/{}", code, reason);
            write!(&mut buffed, "HTTP/1.1 {} {}\n\n", code, reason).expect("Error while writing to output\n");
        }
    }

    println!("end handle_client")
}

fn handle_get<H: ContentHandle>(url: String, gzip_encoding: bool, buffed: &mut Write, manager: &ContentManager<H>) {
    match manager.find_content(&url, gzip_encoding) {
        Some(mut handle) => {
            write!(buffed, "HTTP/1.1 200 OK\n").expect("Error while writing to output\n");

            if handle.is_gzipped() {
                write!(buffed, "Content-Encoding:gzip\n").expect("Error while writing to output\n");
            }
            write!(buffed, "\n").expect("Error while writing to output\n");
            handle.write_to(buffed)
        }
        None => {
            write!(buffed, "HTTP/1.1 404 Not Found\n\n").expect("Error while writing to output\n");
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn read_header_works() {

    }
}