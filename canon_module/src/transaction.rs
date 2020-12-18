use core::marker::PhantomData;

use arrayvec::ArrayVec;
use canonical::{Canon, Sink, Source, Store};

use crate::Q_T_SIZE;

/// Represents the type of a transaction
pub struct Transaction<Over, A, R, const ID: u8> {
    /// Arguments, in form of a tuple or single value
    args: A,
    /// The expected return type
    _return: PhantomData<(Over, R)>,
}

impl<Over, A, R, const ID: u8> Clone for Transaction<Over, A, R, ID>
where
    A: Clone,
{
    fn clone(&self) -> Self {
        Transaction {
            args: self.args.clone(),
            _return: PhantomData,
        }
    }
}

impl<Over, A, R, const N: u8> Transaction<Over, A, R, N> {
    /// Create a new transaction
    pub fn new(args: A) -> Self {
        Transaction {
            args,
            _return: PhantomData,
        }
    }

    /// Returns a reference to the transactions arguments
    pub fn args(&self) -> &A {
        &self.args
    }

    /// Consumes transaction and returns the argument
    pub fn into_args(self) -> A {
        self.args
    }
}

impl<Over, A, R, S, const ID: u8> Canon<S> for Transaction<Over, A, R, ID>
where
    A: Canon<S>,
    S: Store,
{
    fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error> {
        self.args.write(sink)
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
        Ok(Transaction {
            args: A::read(source)?,
            _return: PhantomData,
        })
    }

    fn encoded_len(&self) -> usize {
        self.args.encoded_len()
    }
}

pub struct RawTransaction(ArrayVec<[u8; Q_T_SIZE]>);
