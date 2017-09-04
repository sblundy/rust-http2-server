use std::thread::{spawn, JoinHandle};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::Arc;
use std::sync::Mutex;


type Job = Box<FnBox + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    threads: Vec<Worker>,
    sender: Sender<Message>
}

struct Worker {
    id: usize,
    handle: Option<JoinHandle<()>>
}

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
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &mut self.threads {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.threads {
            println!("Shutting down worker {}", worker.id);
            if let Some(handle) = worker.handle.take() {
                match handle.join() {
                    Ok(_) => {},
                    Err(_) => eprintln!("Error in shutdown")
                }
            }
        }
    }
}

impl Worker {
    fn new(id: usize, rx: Arc<Mutex<Receiver<Message>>>) -> Worker {
        let handle = spawn(move || {
            loop {
                match rx.lock() {
                    Ok(lock) => match lock.recv() {
                        Ok(Message::NewJob(job)) => job.call_box(),
                        Ok(Message::Terminate) => {
                            println!("Terminating worker {}", id);
                            break
                        },
                        Err(e) => eprintln!("Error receiving job:{}", e)
                    },
                    Err(e) => eprintln!("Error receiving lock:{}", e)
                }
            }
        });
        Worker {
            id,
            handle: Some(handle)
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