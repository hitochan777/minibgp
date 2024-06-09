use tracing::{debug, info, instrument};

use crate::config::Config;
use crate::event::Event;
use crate::event_queue::EventQueue;
use crate::state::State;

#[derive(Debug)]
pub struct Peer {
    config: Config,
    state: State,
    event_queue: EventQueue,
}

impl Peer {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            state: State::Idle,
            event_queue: EventQueue::new(),
        }
    }

    #[instrument]
    pub fn start(&mut self) {
        info!("Starting peer");
        self.event_queue.enqueue(Event::ManualStart);
    }

    #[instrument]
    pub async fn next(&mut self) {
        if let Some(event) = self.event_queue.dequeue() {
            info!("Start handling event: {:?}", event);
            self.handle_event(event).await;
        }
    }

    async fn handle_event(&mut self, event: Event) {
        match &self.state {
            State::Idle => match event {
                Event::ManualStart => {
                    self.state = State::Connect;
                }
                _ => {}
            },
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn peer_can_transition_to_connected_state() {
        let config: Config = "64512 127.0.0.1 64513 127.0.0.2 active".parse().unwrap();
        let mut peer = Peer::new(config);
        peer.start();
        peer.next().await;
        assert_eq!(peer.state, State::Connect);
    }
}
