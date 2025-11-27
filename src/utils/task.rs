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
        if let Some(state) = tasks.get_mut(id) {
            if let Some(result) = &*state.final_result.lock().unwrap() {
                let status = match result {
                    Ok(val) => TaskStatus::Done(val.clone()),
                    Err(err) => TaskStatus::Error(err.clone()),
                };
                tasks.remove(id);
                return status;
            }

            if !state.handle.is_finished() {
                TaskStatus::Busy
            } else {
                // The thread is finished, but the result hasn't been moved yet.
                // This can happen in a race condition. Let's try to lock and check again.
                if let Some(result) = state.final_result.lock().unwrap().take() {
                     let status = match result {
                        Ok(val) => TaskStatus::Done(val),
                        Err(err) => TaskStatus::Error(err),
                    };
                    tasks.remove(id);
                    status
                } else {
                    // This should be very rare.
                    TaskStatus::Busy
                }
            }
        } else {
            TaskStatus::NonExistent
        }
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
        tasks.retain(|_id, state| {
            if state.handle.is_finished() {
                // Also check if there's a final result to avoid removing a task that just finished
                // but hasn't been processed by task_status yet. This is a defensive check.
                state.final_result.lock().unwrap().is_none()
            } else {
                true
            }
        });
    }
}

// --- Dettagli Interni ---

struct TaskState<T, E, S> {
    handle: Arc<thread::JoinHandle<()>>,
    output_receiver: mpsc::Receiver<S>,
    final_result: Arc<Mutex<Option<Result<T, E>>>>,
}
