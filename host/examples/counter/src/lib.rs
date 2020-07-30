#![no_std]

use bridge::Bridge;
use canonical::Snap;
use canonical_derive::Canon;

#[derive(Canon)]
struct Counter {
    value: i32,
}

impl Counter {
    pub fn _get_value(&self) -> i32 {
        self.value
    }

    pub fn _increment(&mut self) {
        self.value += 1;
    }

    pub fn _decrement(&mut self) {
        self.value -= 1;
    }
}

impl Counter {
    pub fn get_value<S>(id: &S::Ident) -> Result<i32, S::Error> {
        let slf: Self = S::get(id)?;
        Ok(slf._get_value())
    }

    pub fn increment<S: Store>(id: &S::Ident) -> Result<(S::Ident, ()), S::Error> {
        let slf: Self = S::get(id)?;
        let ret = slf._increment()
        S::put(slf).map(|id| (id, ret))
    }

    pub fn decrement<S: Store>(id: &S::Ident) -> Result<(S::Ident, ()), S::Error> {
        let slf: Self = S::get(id)?;
        let ret = slf._decrement()
        S::put(slf).map(|id| (id, ret))
    }
}

#[no_mangle]
fn it_works() {
    let mut a = Transaction::Increment;
    let _snap = a.snapshot::<Bridge<[u8; 32]>>();
}
