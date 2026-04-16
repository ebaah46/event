// ===========================================================
// File: dispatcher.rs
// Description:
// Author: BEKs <ebaah72@gmail.com>
// Created: 15/04/2026
// ===========================================================

use std::sync::{Arc, RwLock};

use crate::{Event, EventSubscriber};

/**
 * An Event System's dispatcher
 */
#[derive(Debug)]
pub struct EventDispatcher<E: Event> {
    subscribers: RwLock<Vec<Arc<dyn EventSubscriber<E>>>>,
}

impl<E: Event> Default for EventDispatcher<E> {
    fn default() -> Self {
        Self {
            subscribers: Default::default(),
        }
    }
}

impl<E: Event> EventDispatcher<E> {
    pub fn new() -> Self {
        Self {
            subscribers: RwLock::new(vec![]),
        }
    }

    /*
     * A method for all event subscribers to call to register their event handler
     */
    pub fn subscribe(&self, subscriber: Arc<dyn EventSubscriber<E>>) {
        dbg!("Received request to subscribe");
        if let Ok(_) = self
            .subscribers
            .write()
            .map(|mut list| list.push(subscriber))
        {
            dbg!("Registeration completed in dispatcher");
        } else {
            dbg!("Registeration failed in dispatcher");
        }
    }

    /*
     * A method to be triggered to dispatch an event
     */
    pub fn dispatch(&self, event: E) {
        if let Ok(listeners) = self.subscribers.read() {
            for listener in listeners.iter() {
                (*listener.as_ref()).on_event(event.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::{Display, Write};

    use super::*;

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
    }

    impl EventSubscriber<TestEvent> for TestListener {
        fn on_event(&self, event: TestEvent) {
            _ = self
                .event_messages
                .write()
                .map(|mut list| list.push(event.to_string()));
            println!("Event triggered with :{}", event);
        }
    }

    #[test]
    fn test_register_event_subscriber_success() {
        let dispatcher: EventDispatcher<TestEvent> = EventDispatcher::new();
        let subscriber = Arc::new(TestListener::default());
        {
            let dispatcher_gaurd = dispatcher.subscribers.read().unwrap();
            assert_eq!(dispatcher_gaurd.len(), 0);
            let subscriber_gaurd = subscriber.event_messages.read().unwrap();
            assert_eq!(subscriber_gaurd.len(), 0);
        }
        dispatcher.subscribe(subscriber.clone());
        let dispatcher_gaurd = dispatcher.subscribers.read().unwrap();
        assert_eq!(dispatcher_gaurd.len(), 1);
        let subscriber_gaurd = subscriber.event_messages.read().unwrap();
        assert_eq!(subscriber_gaurd.len(), 0);
    }

    #[test]
    fn test_dispatch_event_success() {
        let dispatcher: EventDispatcher<TestEvent> = EventDispatcher::new();
        let subscriber = Arc::new(TestListener::default());
        {
            let dispatcher_gaurd = dispatcher.subscribers.read().unwrap();
            assert_eq!(dispatcher_gaurd.len(), 0);
            let messages = subscriber.event_messages.read().unwrap();
            assert_eq!(messages.len(), 0);
        }
        dispatcher.subscribe(subscriber.clone());
        let dispatcher_gaurd = dispatcher.subscribers.read().unwrap();
        assert_eq!(dispatcher_gaurd.len(), 1);
        {
            let messages = subscriber.event_messages.read().unwrap();
            assert_eq!(messages.len(), 0)
        }
        dispatcher.dispatch(TestEvent::Second);
        let messages = subscriber.event_messages.read().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], TestEvent::Second.to_string());
    }

    #[test]
    fn test_dispatch_event_with_no_listener() {
        let dispatcher: EventDispatcher<TestEvent> = EventDispatcher::new();
        let subscriber = Arc::new(TestListener::default());
        {
            let dispatcher_gaurd = dispatcher.subscribers.read().unwrap();
            assert_eq!(dispatcher_gaurd.len(), 0);
            let messages = subscriber.event_messages.read().unwrap();
            assert_eq!(messages.len(), 0);
        }
        dispatcher.dispatch(TestEvent::Second);
        let messages = subscriber.event_messages.read().unwrap();
        assert_eq!(messages.len(), 0);
    }
}
