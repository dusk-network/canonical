use canonical::{Canon, CanonError, Store};
use canonical_derive::Canon;

#[derive(Clone, Canon)]
pub struct Counter {
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
    pub fn get_value<S: Store>(
        store: S,
        id: &S::Ident,
    ) -> Result<i32, CanonError<S::Error>> {
        let slf: Self = store.get(id)?;
        Ok(slf._get_value())
    }

    pub fn increment<S: Store>(
        store: S,
        id: &S::Ident,
    ) -> Result<(S::Ident, ()), CanonError<S::Error>> {
        let buffer = S::buffer();
        let mut slf: Self = store.get(id)?;
        let ret = slf._increment();
        let len = Canon::<S>::encoded_len(&ret);
        let mut slice = &mut buffer[0..len];
        Canon::<S>::write(&ret, &mut slice)?;
        store.put(slice).map(|id| (id, ret))
    }

    pub fn decrement<S: Store>(
        store: S,
        id: &S::Ident,
    ) -> Result<(S::Ident, ()), CanonError<S::Error>> {
        let buffer = S::buffer();
        let mut slf: Self = store.get(id)?;
        let ret = slf._decrement();
        let len = Canon::<S>::encoded_len(&ret);
        let mut slice = &mut buffer[0..len];
        Canon::<S>::write(&ret, &mut slice)?;
        store.put(slice).map(|id| (id, ret))
    }
}
