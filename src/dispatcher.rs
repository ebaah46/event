// ===========================================================
// File: dispatcher.rs
// Description:
// Author: BEKs <ebaah72@gmail.com>
// Created: 15/04/2026
// ===========================================================

use crossbeam_channel::{Sender, unbounded};
use parking_lot::RwLock;
use std::sync::Arc;
use std::thread;

use crate::{Event, EventSubscriber};

/**
 * An Event System's dispatcher
 */
#[derive(Debug)]
pub struct EventDispatcher<E: Event> {
    subscribers: Arc<RwLock<Vec<Arc<dyn EventSubscriber<E>>>>>,
    sender: Option<Sender<E>>,
}

impl<E: Event> Default for EventDispatcher<E> {
    fn default() -> Self {
        Self {
            subscribers: Default::default(),
            sender: None,
        }
    }
}

impl<E: Event> EventDispatcher<E> {
    pub fn new() -> Self {
        let _ = env_logger::builder().is_test(true).try_init();
        // spin off a thread to receive the events and trigger the listeners
        let subscribers = Arc::new(RwLock::new(vec![]));
        let (tx, rx) = unbounded::<E>();
        let subs_for_dispatcher = Arc::clone(&subscribers);
        let builder = thread::Builder::new().name("Event system thread".into());

        builder
            .spawn(move || {
                for event in rx {
                    log::info!("EventDispatcher - dispatcher thread - received new event");
                    let listeners: Vec<Arc<dyn EventSubscriber<E>>> = {
                        let gaurd = subs_for_dispatcher.read();
                        gaurd.clone()
                    };
                    for listener in listeners.iter() {
                        (*listener.as_ref()).on_event(event.clone());
                    }
                }
            })
            .expect("Failed to launch event system thread");
        log::info!("EventDispatcher - new - created new event system");
        Self {
            subscribers,
            sender: Some(tx),
        }
    }

    /*
     * A method for all event subscribers to call to register their event handler
     */
    pub fn subscribe(&self, subscriber: Arc<dyn EventSubscriber<E>>) {
        log::info!("EventDispatcher - subscribe - added new subscriber");
        self.subscribers.write().push(subscriber);
    }

    /*
     * A method to be triggered to dispatch an event
     */
    pub fn dispatch(&self, event: E) {
        if let Some(sender) = self.sender.clone() {
            let _ = sender.send(event);
            log::info!("EventDispatcher - dispatch - sent new event");
        }
    }

    /*
     * Read the number of subscribers
     */
    pub fn subscriber_length(&self) -> usize {
        self.subscribers.read().len()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::{
        fmt::Display,
        sync::atomic::{AtomicI8, Ordering},
        time::Duration,
    };

    #[derive(Debug, Clone, Copy)]
    enum TestEvent {
        First,
        Second,
        Third,
    }

    impl From<TestEvent> for &'static str {
        fn from(value: TestEvent) -> Self {
            match value {
                TestEvent::First => "first",
                TestEvent::Second => "second",
                TestEvent::Third => "third",
            }
        }
    }

    impl Display for TestEvent {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let s: &'static str = (*self).into();
            f.write_str(s)
        }
    }

    #[derive(Default, Debug)]
    struct TestListener {
        event_messages: RwLock<Vec<String>>,
        data_change: AtomicI8,
    }

    impl EventSubscriber<TestEvent> for TestListener {
        fn on_event(&self, event: TestEvent) {
            self.data_change.store(
                self.data_change.load(Ordering::Relaxed) + 1,
                Ordering::Relaxed,
            );
            self.event_messages.write().push(event.to_string());
        }
    }

    #[test]
    fn test_register_event_subscriber_success() {
        let dispatcher: EventDispatcher<TestEvent> = EventDispatcher::new();
        let subscriber = Arc::new(TestListener::default());
        {
            let dispatcher_gaurd = dispatcher.subscribers.read();
            assert_eq!(dispatcher_gaurd.len(), 0);
            let subscriber_gaurd = subscriber.event_messages.read();
            assert_eq!(subscriber_gaurd.len(), 0);
        }
        dispatcher.subscribe(subscriber.clone());
        let dispatcher_gaurd = dispatcher.subscribers.read();
        assert_eq!(dispatcher_gaurd.len(), 1);
        assert_eq!(subscriber.data_change.load(Ordering::Relaxed), 0);
        let subscriber_gaurd = subscriber.event_messages.read();
        assert_eq!(subscriber_gaurd.len(), 0);
    }

    #[test]
    fn test_dispatch_event_success() {
        let dispatcher: EventDispatcher<TestEvent> = EventDispatcher::new();
        let subscriber = Arc::new(TestListener::default());
        {
            let dispatcher_gaurd = dispatcher.subscribers.read();
            assert_eq!(dispatcher_gaurd.len(), 0);
            assert_eq!(subscriber.data_change.load(Ordering::Relaxed), 0);
            let messages = subscriber.event_messages.read();
            assert_eq!(messages.len(), 0);
        }
        dispatcher.subscribe(subscriber.clone());
        let dispatcher_gaurd = dispatcher.subscribers.read();
        assert_eq!(dispatcher_gaurd.len(), 1);
        {
            assert_eq!(subscriber.data_change.load(Ordering::Relaxed), 0);
            let messages = subscriber.event_messages.read();
            assert_eq!(messages.len(), 0)
        }
        dispatcher.dispatch(TestEvent::Second);
        let mut timeout = 100;
        while subscriber.data_change.load(Ordering::Relaxed) == 0 && timeout > 0 {
            thread::sleep(Duration::from_millis(100));
            timeout -= 1;
        }
        assert!(timeout > 0);
        assert_eq!(subscriber.data_change.load(Ordering::Relaxed), 1);
        {
            let messages = subscriber.event_messages.read();
            assert_eq!(messages.len(), 1);
            assert_eq!(messages[0], TestEvent::Second.to_string());
        }

        // dispatch multiple events
        dispatcher.dispatch(TestEvent::First);
        dispatcher.dispatch(TestEvent::Third);

        timeout = 300;
        while subscriber.data_change.load(Ordering::Relaxed) < 3 && timeout > 0 {
            thread::sleep(Duration::from_millis(100));
            timeout -= 1;
        }
        assert!(timeout > 0);
        assert_eq!(subscriber.data_change.load(Ordering::Relaxed), 3);
        let messages = subscriber.event_messages.read();
        assert_eq!(messages.len(), 3);
        // messages are received in the same order as sent
        assert_eq!(messages[1], TestEvent::First.to_string());
        assert_eq!(messages[2], TestEvent::Third.to_string());
    }

    #[test]
    fn test_dispatch_event_with_no_listener() {
        let dispatcher: EventDispatcher<TestEvent> = EventDispatcher::new();
        let subscriber = Arc::new(TestListener::default());
        {
            let dispatcher_gaurd = dispatcher.subscribers.read();
            assert_eq!(dispatcher_gaurd.len(), 0);
            let messages = subscriber.event_messages.read();
            assert_eq!(messages.len(), 0);
        }
        dispatcher.dispatch(TestEvent::Second);
        let messages = subscriber.event_messages.read();
        assert_eq!(messages.len(), 0);
    }
}
