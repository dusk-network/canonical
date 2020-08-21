use std::rc::Rc;

use std::cell::RefCell;

use crate::{Canon, CanonError, Sink, Source, Store};

#[derive(Clone, Debug)]
/// A Handle to a value that is either local or in storage
pub enum Handle<T, S: Store> {
    /// Value is kept reference counted locally
    Value {
        /// The reference counted value on the heap
        rc: Rc<T>,
        /// The cached identity of the value, if any
        cached_ident: RefCell<Option<Box<S::Ident>>>,
    },
    /// Value is represented by it's Identity
    Ident {
        /// The value identity
        ident: S::Ident,
        /// The store where to request the value
        store: S,
    },
}

impl<T, S> Canon<S> for Handle<T, S>
where
    S: Store,
    T: Canon<S>,
{
    fn write(
        &self,
        sink: &mut impl Sink<S>,
    ) -> Result<(), CanonError<S::Error>> {
        match self {
            Handle::Value { rc, cached_ident } => {
                let len = (**rc).encoded_len();
                let ident_len = S::Ident::default().as_ref().len();
                if len <= ident_len {
                    // inline value
                    Canon::<S>::write(&mut (len as u8), sink)?;
                    Canon::<S>::write(&**rc, sink)?;
                } else {
                    Canon::<S>::write(&mut 0u8, sink)?;
                    let mut subsink = sink.recur();
                    Canon::<S>::write(&**rc, &mut subsink)?;
                    let ident = subsink.fin()?;

                    *cached_ident.borrow_mut() = Some(Box::new(ident.clone()));
                    sink.copy_bytes(&ident.as_ref());
                }
            }
            Handle::Ident { ref ident, .. } => {
                Canon::<S>::write(&mut 0u8, sink)?;
                sink.copy_bytes(&ident.as_ref());
            }
        }
        Ok(())
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, CanonError<S::Error>> {
        let len = u8::read(source)?;
        if len > 0 {
            // inlined value
            Ok(Handle::Value {
                rc: Rc::new(T::read(source)?),
                cached_ident: RefCell::new(None),
            })
        } else {
            // ident
            let mut ident = S::Ident::default();
            let slice = ident.as_mut();
            let bytes = source.read_bytes(slice.len());
            slice.copy_from_slice(bytes);
            Ok(Handle::Ident {
                ident,
                store: source.store(),
            })
        }
    }

    fn encoded_len(&self) -> usize {
        let ident_len = S::Ident::default().as_ref().len();
        match &self {
            Handle::Value { rc, .. } => {
                // If the length is larger nthan `ident_len` + 1,
                // The value will not be inlined, and saved as tag + ident
                1 + core::cmp::max(rc.encoded_len() as usize, ident_len)
            }
            Handle::Ident { .. } => 1 + ident_len,
        }
    }
}

impl<T, S> Handle<T, S>
where
    S: Store,
    T: Canon<S>,
{
    /// Construct a new `Handle` from value `t`
    pub fn new(t: T) -> Self {
        Handle::Value {
            rc: Rc::new(t),
            cached_ident: RefCell::new(None),
        }
    }

    /// Returns the value behind the `Handle`
    pub fn restore(&self) -> Result<T, CanonError<S::Error>> {
        match &self {
            Handle::Value { rc, .. } => Ok((**rc).clone()),
            Handle::Ident { ident, store } => store.get(ident),
        }
    }

    /// Commits the value to the store
    pub fn commit(&mut self, _store: &S) -> Result<(), CanonError<S::Error>> {
        match self {
            Handle::Ident { .. } => (),
            Handle::Value { rc: _, cached_ident } => {
                match *cached_ident.borrow() {
                    Some(_) => unimplemented!(),
                    None => (),
                }
            }
        }
        //unimplemented!()
        Ok(())
    }
}
