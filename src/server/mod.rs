use std::net::{TcpListener};
use std::path::Path;
use std::thread::spawn;
use std::sync::Arc;
use self::file_system::{FileSystemAdapter};
use self::handlers::handle_client;

pub fn serve(listener: TcpListener, root: &Path) {
    let fs_adapter = FileSystemAdapter::new(root);
    let adapter_rc = Arc::new(fs_adapter);

    // accept connections and process them in separate threads
    for stream_ref in listener.incoming() {
        match stream_ref {
            Ok(stream) => {
                let local_rc = adapter_rc.clone();
                spawn(move || handle_client(stream, local_rc.as_ref()));
            },
            Err(e) => println!("Error with stream:{}", e)
        }
    }
}

mod http;
mod file_system;
mod content_manager;
mod handlers;