#![no_std]

use bridge::Bridge;
use canonical::{Canon, Sink, Snap};
// use canonical_derive::Canon;

#[no_mangle]
fn it_works() {
    let mut a = 3u64;
    let snap = a.snapshot::<Bridge<[u8; 32]>>();
}
