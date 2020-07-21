#![no_std]

use bridge::Bridge;
use canonical::Snap;
use canonical_derive::Canon;

#[derive(Canon)]
enum Transaction {
    Increment,
    Decrement,
}

#[no_mangle]
fn it_works() {
    let mut a = Transaction::Increment;
    let _snap = a.snapshot::<Bridge<[u8; 32]>>();
}
