#[cfg(feature = "std")]
mod handle {
    use crate::{Canon, ConstantLength, InvalidEncoding, Sink, Source};

    use std::rc::Rc;

    #[derive(Debug)]
    pub enum Handle<T, S: Store> {
        Rc(Rc<T>),
        Ident(S::Ident, Arc<S>),
        Inline(S::Ident),
    }

    impl<T, S> Canon for Handle<T, S>
    where
        S: for<'a> Store<'a>,
    {
        fn write(&self, _sink: &mut impl Sink) {
            unimplemented!()
        }

        fn read(_source: &mut impl Source) -> Result<Self, InvalidEncoding> {
            unimplemented!()
        }
    }

    impl<T, S> ConstantLength for Handle<T, S> {
        const LEN: usize = S::Ident + 1;
    }

    impl<T, S> Handle<T, S> {
        pub fn new(t: T) -> Self {
            unimplemented!();
        }
    }
}

#[cfg(not(feature = "std"))]
mod handle {
    use crate::{Canon, ConstantLength, InvalidEncoding, Sink, Source, Store};

    use core::marker::PhantomData;

    #[derive(Debug)]
    pub enum Handle<T, S> {
        Ident(S),
        Inline(PhantomData<T>),
    }

    impl<T, S> Canon for Handle<T, S>
    where
        S: for<'a> Store<'a>,
    {
        fn write(&self, _sink: &mut impl Sink) {
            unimplemented!()
        }

        fn read(_source: &mut impl Source) -> Result<Self, InvalidEncoding> {
            unimplemented!()
        }
    }

    impl<T, S> ConstantLength for Handle<T, S>
    where
        S: for<'a> Store<'a>,
    {
        const LEN: usize = S::Ident::LEN + 1;
    }

    impl<T, S> Handle<T, S> {
        pub fn new(_t: T) -> Self {
            unimplemented!();
        }
    }
}

pub use handle::Handle;
