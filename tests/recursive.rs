use canon::{Canon, Handle, InvalidEncoding, Sink, Store};
use canon_derive::Canon;

mod toy_store;
use toy_store::ToyStore;

#[derive(Canon, PartialEq, Debug)]
enum LinkedList<I, T> {
    Empty,
    Node { value: T, next: Handle<I, Self> },
}
