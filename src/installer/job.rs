use super::{Event, Plan, run};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
};
pub struct Job {
    pub events: Receiver<Event>,
    cancel: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}
impl Job {
    pub fn start(plan: Plan, notify: impl Fn() + Send + 'static) -> Self {
        let (sender, events) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        let worker = thread::spawn(move || {
            let progress = |message: String, bytes, total| {
                let _ = sender.send(Event::Progress {
                    message,
                    bytes,
                    total,
                });
                notify();
            };
            let result = run(&plan, &flag, &progress);
            let _ = sender.send(Event::Finished(result));
            notify();
        });
        Self {
            events,
            cancel,
            worker: Some(worker),
        }
    }
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancel();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
