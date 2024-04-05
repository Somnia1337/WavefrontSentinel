use std::sync::mpsc::Receiver;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

/// Represents HTTP status codes used in HTTP response messages.
pub enum HttpStatusCode {
    Ok,
    NotFound,
    BadRequest,
    InternalServerError,
}

impl HttpStatusCode {
    /// Returns the status line corresponding to the HTTP status code.
    pub fn status_line(&self) -> &'static str {
        match self {
            HttpStatusCode::Ok => "200 OK",
            HttpStatusCode::NotFound => "404 NOT FOUND",
            HttpStatusCode::BadRequest => "400 BAD REQUEST",
            HttpStatusCode::InternalServerError => "500 INTERNAL SERVER ERROR",
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let msg = receiver.lock().unwrap().recv();
            match msg {
                Ok(job) => {
                    println!("worker {id} got a job, executing...");
                    job();
                }
                Err(_) => {
                    println!("worker {id} shutting down...");
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Constructs a new, filled thread pool with the specified capacity.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity is 0.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

// todo: test module
