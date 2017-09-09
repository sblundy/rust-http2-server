use std::io::{Read, Write};
use bufstream::BufStream;
use chrono::{FixedOffset,DateTime, Utc};
use super::content_manager::{ContentHandle, ContentManager};
use super::http::{parse_request, Request, BadRequest};

pub fn handle_client<H: ContentHandle, S: Read + Write>(stream: S, manager: &ContentManager<H>) {
    println!("in handle_client");

    let mut buffed = BufStream::new(stream);
    let request = parse_request(&mut buffed);

    match request {
        Ok(Request::Get(url, headers)) => {
            handle_get(url, headers.accept_encoding_gzip(), headers.if_modified_since(), false, &mut buffed, manager);
        }
        Ok(Request::Head(url, headers)) => {
            handle_get(url, headers.accept_encoding_gzip(), headers.if_modified_since(), true, &mut buffed, manager);
        }
        Ok(Request::Options(url, _)) => {
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

fn handle_options<H: ContentHandle>(url_op: Option<String>, buffed: &mut Write, manager: &ContentManager<H>) {
    match url_op {
        None => {
            let headers: Vec<(&str, &str)> = vec![
                ("Allow", "OPTIONS, GET, HEAD"),
                ("Content-Length", "0")
            ];
            let handler: Option<H> = None;
            write_response(buffed, "200", "OK", headers, handler);
        }
        Some(url) => {
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

#[cfg(test)]
mod tests {
    #[test]
    fn read_header_works() {

    }
}