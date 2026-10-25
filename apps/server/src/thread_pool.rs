use std::{
  sync::{Arc, Mutex, mpsc},
  thread::{self, JoinHandle},
};

struct Worker {
  thread: JoinHandle<()>,
  id: usize,
}

impl Worker {
  pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
    let thread_builder = thread::Builder::new();
    let thread = thread_builder
      .name(format!("Thread {id}"))
      .spawn(move || {
        loop {
          let job = receiver.lock().unwrap().recv();

          match job {
            Ok(job) => job(),
            Err(_) => println!("Worker {id} disconnected"),
          };
        }
      })
      .unwrap();

    Worker { thread, id }
  }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
  workers: Vec<Worker>,
  sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
  pub fn new(num_threads: usize) -> ThreadPool {
    let mut workers: Vec<Worker> = Vec::with_capacity(num_threads);
    let (sender, receiver) = mpsc::channel::<Job>();
    let receiver = Arc::new(Mutex::new(receiver));

    for i in 0..num_threads {
      let receiver = receiver.clone();
      let worker = Worker::new(i, receiver);

      workers.push(worker);
    }

    ThreadPool {
      workers,
      sender: Some(sender),
    }
  }

  pub fn execute<J>(&self, job: J)
  where
    J: FnOnce() + Send + 'static,
  {
    self.sender.as_ref().unwrap().send(Box::new(job)).unwrap();
  }
}

impl Drop for ThreadPool {
  fn drop(&mut self) {
    for worker in self.workers.drain(..) {
      println!("Shutting down worker {}", worker.id);

      if let Err(error) = worker.thread.join() {
        eprintln!(
          "Failed to gracefully shutdown worker {}. Error: {:#?}",
          worker.id, error
        );
      };
    }
  }
}
