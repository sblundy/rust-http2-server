use std::thread::{spawn, JoinHandle};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::Arc;
use std::sync::Mutex;


pub struct ThreadPool {
    threads: Vec<Worker>,
    sender: Sender<Job>
}

struct Worker {
    id: usize,
    handle: JoinHandle<()>
}

type Job = Box<FnBox + Send + 'static>;

impl ThreadPool {
    pub fn new(num: usize) -> ThreadPool {
        assert!(num > 0);
        let mut threads = Vec::with_capacity(num);
        let (tx, rx) = mpsc::channel();
        let rx_rc = Arc::new(Mutex::new(rx));

        for id in 0..num {
            threads.push(Worker::new(id, rx_rc.clone()));
        }
        ThreadPool {
            threads,
            sender: tx
        }
    }

    pub fn execute<F>(&self, f: F)
        where F: FnOnce() + Send + 'static {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

impl Worker {
    fn new(id: usize, rx: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let handle = spawn(move || {
            loop {
                match rx.lock() {
                    Ok(lock) => match lock.recv() {
                        Ok(job) => job.call_box(),
                        Err(e) => println!("Error receiving job:{}", e)
                    },
                    Err(e) => println!("Error receiving lock:{}", e)
                }
            }
        });
        Worker {
            id,
            handle
        }
    }
}
trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}