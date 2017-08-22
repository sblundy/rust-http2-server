use std::net::{TcpListener};
use std::path::Path;
use self::file_system::{FileSystemAdapter};
use self::handlers::handle_client;

pub fn serve(listener: TcpListener, root: &Path) {
    let fs_adapter = FileSystemAdapter::new(root);

    // accept connections and process them serially
    for stream_ref in listener.incoming() {
        match stream_ref {
            Ok(stream) => handle_client(stream, &fs_adapter),
            Err(e) => println!("Error with stream:{}", e)
        }
    }
}

mod http;
mod file_system;
mod content_manager;
mod handlers;