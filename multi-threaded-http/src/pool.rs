use std::{
	sync::{
		mpsc::{channel, Receiver, Sender},
		Arc, Mutex,
	},
	thread::{self, JoinHandle},
};

// Send typ means the jobs should be able to move around in the threads?
type Runnable = Box<dyn FnOnce() + Send + 'static>;

// Job is either a Task or Stop (waiting to stop)
// If it is a task, then it has an associated function
enum Job {
	Task(Runnable),
	Stop,
}

pub struct ThreadPool {
	sender: Sender<Job>,  // a channel to send work to worker
	workers: Vec<Worker>, // a bunch of worker threads
}

impl ThreadPool {
	pub fn new(size: usize) -> ThreadPool {
		let (sender, receiver) = channel();
		let receiver = Arc::new(Mutex::new(receiver)); // goes to the worker

		let mut workers = Vec::with_capacity(size);

		for _ in 0..size {
			// every worker gets a copy of the receiver associated with the sender that is sending work
			let worker = Worker::new(Arc::clone(&receiver));
			workers.push(worker);
		}

		ThreadPool { sender, workers }
	}

	// sending job to a thread?
	pub fn submit<F>(&mut self, f: F)
	where
		F: FnOnce() + Send + 'static,
	{
		let job = Job::Task(Box::new(f)); // create a new job
		self.sender.send(job).unwrap(); // send job to threadpool
	}

	pub fn size(&self) -> usize {
		self.workers.len()
	}
}

impl Drop for ThreadPool {
	fn drop(&mut self) {
		for _ in 0..self.workers.len() {
			self.sender.send(Job::Task).unwrap(); // sending Stop to all threads in the pool
		}

		for worker in &mut self.workers {
			if let Some(thread) = worker.thread.take() {
				// take the memory, but why?
				thread.join().unwrap(); // let's wait for worker to be defined
			}
		}
	}
}

struct Worker {
	// a uniquely owned permission to join a thread, there is no other way to join the thread for which this thread is
	thread: Option<JoinHandle<()>>,
}

impl Worker {
	fn new(receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
		// spawn a worker thread
		let thread = std::thread::spawn(move || {
			loop {
				let job = receiver.lock().unwrap().recv();
				match job {
					Ok(Job::Task(f)) => f(),
					Ok(Job::Stop) => break,
					Err(_) => break,
				}
			}
		});

		Worker {
			thread: Some(thread),
		}
	}
}
