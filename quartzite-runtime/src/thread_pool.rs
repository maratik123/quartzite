//! Fixed-size worker thread pool for background task execution.
use std::{
    sync::Mutex,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

type Task = Box<dyn FnOnce() + Send>;

/// Fixed-size worker thread pool.
///
/// Workers pick up tasks from a shared channel. On `Drop`, the channel is
/// closed and all workers are joined (graceful shutdown — in-flight tasks
/// complete before the pool is destroyed).
///
/// # Examples
///
/// ```no_run
/// use quartzite_runtime::ThreadPool;
///
/// let pool = ThreadPool::new(2);
/// pool.spawn(|| println!("background work"));
/// ```
pub struct ThreadPool {
    sender: Option<Sender<Task>>,
    workers: Vec<JoinHandle<()>>,
}

impl ThreadPool {
    /// Creates a pool with `size` worker threads.
    ///
    /// # Parameters
    ///
    /// - `size`: number of worker threads to spawn; must be at least 1.
    ///
    /// # Panics
    ///
    /// Panics if `size` is 0.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ThreadPool;
    ///
    /// let pool = ThreadPool::new(4);
    /// pool.spawn(|| println!("hello from worker"));
    /// ```
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "thread pool size must be at least 1");
        let (sender, receiver) = mpsc::channel::<Task>();
        let receiver = Mutex::new(receiver);
        let receiver = std::sync::Arc::new(receiver);
        let mut workers = Vec::with_capacity(size);
        for _ in 0..size {
            let rx: std::sync::Arc<Mutex<Receiver<Task>>> = std::sync::Arc::clone(&receiver);
            workers.push(thread::spawn(move || {
                loop {
                    let task = rx.lock().unwrap().recv();
                    match task {
                        Ok(f) => f(),
                        Err(_) => break, // sender dropped — shut down
                    }
                }
            }));
        }
        Self {
            sender: Some(sender),
            workers,
        }
    }

    /// Submits a task. Returns immediately; the task runs on a worker thread.
    ///
    /// # Parameters
    ///
    /// - `f`: closure to execute on a worker thread.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use quartzite_runtime::ThreadPool;
    ///
    /// let pool = ThreadPool::new(2);
    /// pool.spawn(|| println!("running on a worker thread"));
    /// ```
    pub fn spawn(&self, f: impl FnOnce() + Send + 'static) {
        if let Some(s) = &self.sender {
            s.send(Box::new(f)).ok();
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Drop the sender to close the channel so workers exit their receive loop.
        drop(self.sender.take());
        for worker in self.workers.drain(..) {
            worker.join().ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    #[test]
    fn tasks_execute_on_workers() {
        let pool = ThreadPool::new(2);
        let results: Arc<Mutex<Vec<u32>>> = Arc::new(Mutex::new(Vec::new()));

        for i in 0u32..4 {
            let r = Arc::clone(&results);
            pool.spawn(move || r.lock().unwrap().push(i));
        }

        // Give workers time to finish.
        thread::sleep(Duration::from_millis(50));
        drop(pool); // joins workers

        let mut v = results.lock().unwrap().clone();
        v.sort_unstable();
        assert_eq!(v, vec![0, 1, 2, 3]);
    }

    #[test]
    fn drop_joins_workers_gracefully() {
        let pool = ThreadPool::new(2);
        let flag = Arc::new(Mutex::new(false));
        let f = Arc::clone(&flag);
        pool.spawn(move || {
            thread::sleep(Duration::from_millis(20));
            *f.lock().unwrap() = true;
        });
        drop(pool); // must block until task completes
        assert!(*flag.lock().unwrap());
    }
}
