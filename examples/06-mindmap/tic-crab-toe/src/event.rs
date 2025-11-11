use crossterm::event::KeyEvent;
use std::sync::mpsc;

/// Represents all possible events in the application.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppEvent {
    /// Represents a periodic application tick, useful for animations or timed events.
    Tick,
    /// Encapsulates keyboard input events.
    Input(KeyEvent),
}

/// A small event handler that manages the event loop.
///
/// This is a simple wrapper around `std::sync::mpsc` that allows for sending and receiving `AppEvent`s.
pub struct EventHandler {
    /// Event sender channel.
    sender: mpsc::Sender<AppEvent>,
    /// Event receiver channel.
    receiver: mpsc::Receiver<AppEvent>,
    /// Event handler thread.
    handler: std::thread::JoinHandle<()>,
}

impl EventHandler {
    /// Constructs a new `EventHandler`.
    pub fn new(tick_rate: std::time::Duration) -> Self {
        let (sender, receiver) = mpsc::channel();
        let handler = {
            let sender = sender.clone();
            std::thread::spawn(move || {
                let mut last_tick = std::time::Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or_else(|| std::time::Duration::from_secs(0));

                    if crossterm::event::poll(timeout).expect("unable to poll event") {
                        match crossterm::event::read().expect("unable to read event") {
                            crossterm::event::Event::Key(e) => sender.send(AppEvent::Input(e)).expect("failed to send event"),
                            _ => {}
                        }
                    }

                    if last_tick.elapsed() >= tick_rate {
                        sender.send(AppEvent::Tick).expect("failed to send event");
                        last_tick = std::time::Instant::now();
                    }
                }
            })
        };
        Self {
            sender,
            receiver,
            handler,
        }
    }

    /// Receives the next event from the handler thread.
    ///
    /// This function will block until an event is received.
    pub fn next(&self) -> AppEvent {
        self.receiver.recv().expect("failed to receive event")
    }
}
