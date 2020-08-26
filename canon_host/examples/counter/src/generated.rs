#![no_std]
// use canonical::{Canon, CanonError, BridgeStore, Store};
// use canonical_derive::Canon;

use canonical_host::Remote;

trait CounterInterface {
    fn increment(&mut self);
    fn decrement(&mut self);
    fn query(&self) -> i32;
}

impl CounterInterface for Remote<S> {
    fn increment(&mut self) {

    }

    fn decrement(&mut self) {

    }

    fn query(&self) -> i32 {

    }
}
