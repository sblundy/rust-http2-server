use std::net::{TcpListener};
use std::path::Path;
use std::sync::Arc;
use self::file_system::{FileSystemAdapter};
use self::handlers::handle_client;
use self::pool::ThreadPool;

pub fn serve(listener: TcpListener, root: &Path) {
    let fs_adapter = FileSystemAdapter::new(root);
    let adapter_rc = Arc::new(fs_adapter);
    let pool = ThreadPool::new(4);

    // accept connections and process them in separate threads
    for stream_ref in listener.incoming() {
        match stream_ref {
            Ok(stream) => {
                let local_rc = adapter_rc.clone();
                pool.execute(move || handle_client(stream, local_rc.as_ref()));
            },
            Err(e) => println!("Error with stream:{}", e)
        }
    }
}

mod http;
mod file_system;
mod content_manager;
mod handlers;
mod pool;