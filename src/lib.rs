use std::{
    sync::{mpsc, mpsc::Receiver, Arc, Mutex},
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

/// Represents HTTP status codes used in HTTP response messages.
pub enum HttpStatusCode {
    Ok,
    BadRequest,
    NotFound,
    InternalServerError,
}

impl HttpStatusCode {
    /// Returns the status line corresponding to the HTTP status code.
    pub fn status_line(&self) -> &str {
        match self {
            HttpStatusCode::Ok => "200 OK",
            HttpStatusCode::BadRequest => "400 BAD REQUEST",
            HttpStatusCode::NotFound => "404 NOT FOUND",
            HttpStatusCode::InternalServerError => "500 INTERNAL SERVER ERROR",
        }
    }
}

pub enum HttpContentType {
    Html,
    Css,
    Jpg,
    Png,
}

impl HttpContentType {
    pub fn content_type(&self) -> &str {
        match self {
            HttpContentType::Html => "text/html",
            HttpContentType::Css => "text/css",
            HttpContentType::Jpg => "image/jpg",
            HttpContentType::Png => "image/png",
        }
    }
}

impl From<&str> for HttpContentType {
    fn from(extension: &str) -> Self {
        match extension {
            "html" => HttpContentType::Html,
            "css" => HttpContentType::Css,
            "jpg" => HttpContentType::Jpg,
            "jpeg" => HttpContentType::Jpg,
            "png" => HttpContentType::Png,
            _ => HttpContentType::Png, // 默认为 PNG 类型
        }
    }
}

/// Represents a working thread.
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Constructs a new `Worker`, with a thread spawned.
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            match receiver.lock().unwrap().recv() {
                Ok(job) => {
                    println!("> Worker {id} got a job, executing...");
                    job();
                }
                Err(_) => {
                    println!("> Worker {id} shutting down...");
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

/// Represents a pool, containing several threads and a sender.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Constructs a new, filled thread pool with the specified capacity.
    ///
    /// # Panics
    ///
    /// Panics if the specified capacity is 0.
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

    /// Adds a new `job` to the `ThreadPool`.
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
            println!("> Shutting down worker {}", worker.id);
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
