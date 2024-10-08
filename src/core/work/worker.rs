use crate::models::event::Event;

use super::{work::Work, work_handler::WorkEvent};
use std::{
    sync::{atomic::AtomicBool, mpsc::Sender, Arc},
    thread::{self, JoinHandle},
};

/// Represents a worker
pub(super) struct Worker {
    id: usize,
    work_tx: Sender<WorkEvent>,
    pub work: Box<dyn Work + Send>,
    pub tx: Sender<Event>,
    should_stop: Arc<AtomicBool>,
}

impl Worker {
    pub fn new(
        id: usize,
        work_tx: Sender<WorkEvent>,
        tx: Sender<Event>,
        should_stop: Arc<AtomicBool>,
        work: Box<dyn Work + Send>,
    ) -> Worker {
        Worker {
            id,
            work_tx,
            tx,
            work,
            should_stop,
        }
    }
    /// A Helper function to create a new thread for a worker
    /// This function consumes self so we can avoid cloning info to the thread created
    pub fn run_on_separate_thread(self) -> JoinHandle<()> {
        return thread::spawn(move || {
            self.work.run(self.tx, self.should_stop);
            let _ = self.work_tx.send(WorkEvent::Done(self.id));
        });
    }
}
