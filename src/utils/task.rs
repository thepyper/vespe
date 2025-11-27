// src/utils/task.rs

use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use uuid::Uuid;

// --- API Pubblica ---

#[derive(Debug)]
pub enum TaskStatus<T, E> {
    NonExistent,
    Busy,
    Done(T),
    Error(E),
}

#[derive(Clone)]
pub struct TaskManager<T, E, S>
where
    T: Send + 'static,
    E: Send + 'static,
    S: Send + 'static,
{
    tasks: Arc<Mutex<HashMap<Uuid, TaskState<T, E, S>>>>,
}

impl<T, E, S> Default for TaskManager<T, E, S>
where
    T: Send + 'static,
    E: Send + 'static,
    S: Send + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, E, S> TaskManager<T, E, S>
where
    T: Send + 'static,
    E: Send + 'static,
    S: Send + 'static,
{
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn start_task<F>(&self, id: Uuid, task: F)
    where
        F: FnOnce(mpsc::Sender<S>) -> Result<T, E> + Send + 'static,
    {
        let (output_tx, output_rx) = mpsc::channel();
        let final_result = Arc::new(Mutex::new(None));
        let final_result_clone = final_result.clone();

        let handle = thread::spawn(move || {
            let result = task(output_tx);
            let mut final_result_lock = final_result_clone.lock().unwrap();
            *final_result_lock = Some(result);
        });

        let state = TaskState {
            handle: Arc::new(handle),
            output_receiver: output_rx,
            final_result,
        };

        let mut tasks = self.tasks.lock().unwrap();
        tasks.insert(id, state);
    }

    pub fn task_status(&self, id: &Uuid) -> TaskStatus<T, E>
    where
        T: Clone,
        E: Clone,
    {
        let mut tasks = self.tasks.lock().unwrap();
        
        let status = if let Some(state) = tasks.get(id) {
            let result_lock = state.final_result.lock().unwrap();
            if let Some(result) = &*result_lock {
                // Task is finished and has a result.
                match result {
                    Ok(val) => TaskStatus::Done(val.clone()),
                    Err(err) => TaskStatus::Error(err.clone()),
                }
            } else if state.handle.is_finished() {
                // The thread is finished, but no result was written.
                // This indicates a panic within the task.
                // We'll treat this as "Done" so it gets removed, and return a special error.
                // Since we can't change the error type E, we will just remove it
                // and the next call will be NonExistent. For the user, it's like it vanished.
                // A better API would have a `Panicked` status.
                TaskStatus::NonExistent // Treat as gone.
            } else {
                // Not finished yet.
                TaskStatus::Busy
            }
        } else {
            TaskStatus::NonExistent
        };

        // If the task is no longer busy, remove it from the map.
        match status {
            TaskStatus::Busy => (),
            _ => {
                tasks.remove(id);
            }
        }

        status
    }

    pub fn poll_output(&self, id: &Uuid) -> Option<Vec<S>> {
        let tasks = self.tasks.lock().unwrap();
        if let Some(state) = tasks.get(id) {
            let mut all_output = Vec::new();
            while let Ok(output) = state.output_receiver.try_recv() {
                all_output.push(output);
            }
            if all_output.is_empty() {
                None
            } else {
                Some(all_output)
            }
        } else {
            None
        }
    }

    pub fn wait_output(&self, id: &Uuid) -> Option<Vec<S>> {
        let tasks = self.tasks.lock().unwrap();
        if let Some(state) = tasks.get(id) {
            // Block until the first message is received.
            match state.output_receiver.recv() {
                Ok(first_output) => {
                    let mut all_output = vec![first_output];
                    // Drain any other pending messages non-blockingly.
                    while let Ok(output) = state.output_receiver.try_recv() {
                        all_output.push(output);
                    }
                    Some(all_output)
                }
                Err(_) => {
                    // Channel is closed, task is done.
                    None
                }
            }
        } else {
            None
        }
    }
    
    pub fn cleanup(&self) {
        let mut tasks = self.tasks.lock().unwrap();
        // Retain a task if it is not finished, OR if it is finished and has a result.
        // This effectively removes only the tasks that have panicked (finished without a result).
        tasks.retain(|_id, state| {
            !state.handle.is_finished() || state.final_result.lock().unwrap().is_some()
        });
    }
}

// --- Dettagli Interni ---

struct TaskState<T, E, S> {
    handle: Arc<thread::JoinHandle<()>>,
    output_receiver: mpsc::Receiver<S>,
    final_result: Arc<Mutex<Option<Result<T, E>>>>,
}
