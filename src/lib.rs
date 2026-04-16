// ===========================================================
// File: lib.rs
// Description:
// Author: BEKs <ebaah72@gmail.com>
// Created: 15/04/2026
// ===========================================================

use std::fmt::Debug;

pub mod dispatcher;

// all events must be cloneable and be shared across threads
pub trait Event: Clone + Send + 'static {}

// automatic impl of Event for any type that implements clone, send and 'static
impl<T: Clone + Send + 'static> Event for T {}

// all event subscriber must fufill this
pub trait EventSubscriber<E>: Send + Sync + Debug {
    fn on_event(&self, event: E);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
