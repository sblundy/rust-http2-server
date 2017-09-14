extern crate rust_https_server;
extern crate reqwest;
extern crate hyper;

use rust_https_server::start_server;
use reqwest::{Client, StatusCode, Method};
use hyper::header::Allow;
use std::io::Read;


#[test]
fn downloads_get_request() {
    let handle = start_server("test_resources", "127.0.0.1", "0", None).unwrap();

    let client = Client::new().unwrap();
    let url = format!("http://127.0.0.1:{}/index.html", handle.port);
    let mut request = client.get(&url).unwrap();
    match request.send() {
        Ok(response) => assert_eq!(StatusCode::Ok, response.status()),
        Err(e) => panic!("Request error{}", e)
    }
}

#[test]
fn returns_404_on_unknown_file() {
    let handle = start_server("test_resources", "127.0.0.1", "0", None).unwrap();

    let client = Client::new().unwrap();
    let url = format!("http://127.0.0.1:{}/not-index.html", handle.port);
    let mut request = client.get(&url).unwrap();
    match request.send() {
        Ok(response) => assert_eq!(StatusCode::NotFound, response.status()),
        Err(e) => panic!("Request error{}", e)
    }
}

#[test]
fn head_returns_get_request_headers() {
    let handle = start_server("test_resources", "127.0.0.1", "0", None).unwrap();

    let client = Client::new().unwrap();
    let url = format!("http://127.0.0.1:{}/index.html", handle.port);
    let mut request = client.head(&url).unwrap();
    match request.send() {
        Ok(mut response) => {
            assert_eq!(StatusCode::Ok, response.status());
            let mut dummy = String::new();
            assert_eq!(0, response.read_to_string(&mut dummy).unwrap())
        },
        Err(e) => panic!("Request error{}", e)
    }
}

#[test]
fn options_includes_allow_header() {
    let handle = start_server("test_resources", "127.0.0.1", "0", None).unwrap();

    let client = Client::new().unwrap();
    let url = format!("http://127.0.0.1:{}/index.html", handle.port);
    let mut request = client.request(Method::Options, &url).unwrap();
    match request.send() {
        Ok(response) => {
            assert_eq!(StatusCode::Ok, response.status());
            assert_eq!(true, response.headers().has::<Allow>())
        },
        Err(e) => panic!("Request error{}", e)
    }
}
