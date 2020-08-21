#![no_std]
use canonical::{Canon, CanonError, BridgeStore, Store};
use canonical_derive::Canon;

#[derive(Clone, Canon)]
pub struct Counter {
    value: i32,
}

impl Counter {
    pub fn _get_value(&self) -> i32 {
        self.value
    }

    pub fn _adjust(&mut self, by: i32) {
        self.value += by;
    }
}

// impl Counter {
//     pub fn get_value(
//         id: &S::Ident,
//     ) -> Result<i32, CanonError<S::Error>> {
//         let slf: Self = store.get(id)?;
//         Ok(slf._get_value())
//     }

//     pub fn adjust<S: Store>(
//         store: S,
//         id: &S::Ident,
//         by: i32,
//     ) -> Result<(S::Ident, ()), CanonError<S::Error>> {
//         let buffer = S::buffer();
//         let mut slf: Self = store.get(id)?;
//         let ret = slf._adjust(by);
//         let len = Canon::<S>::encoded_len(&ret);
//         let mut slice = &mut buffer[0..len];
//         Canon::<S>::write(&ret, &mut slice)?;
//         store.put(slice).map(|id| (id, ret))
//     }
// }
type Id = [u8; 32];
type St = BridgeStore<Id>;

#[no_mangle]
fn call(mut state: &[u8], args: &[u8]) -> Result<Id, CanonError<<St as Store>::Error>> {
    let mut this: Counter = Canon::<St>::read(&mut state)?;
    this._adjust(1);

    let store = BridgeStore::<Id>::new();
    let ident = store.stash(this);

    Ok(ident)
}
