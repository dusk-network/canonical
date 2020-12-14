use crate::{Canon, Sink, Source, Store};
use core::marker::PhantomData;

/// Represents the type of a query
///
/// `Over` is the type that the query is expected to operate over.
#[derive(Debug)]
pub struct Query<Over, A, R, const ID: u8> {
    /// Arguments, in form of a tuple or single value
    args: A,
    /// The expected return type
    _return: PhantomData<(Over, R)>,
}

impl<Over, A, R, const ID: u8> Clone for Query<Over, A, R, ID>
where
    A: Clone,
{
    fn clone(&self) -> Self {
        Query {
            args: self.args.clone(),
            _return: PhantomData,
        }
    }
}

impl<Over, A, R, const ID: u8> Query<Over, A, R, ID> {
    /// Construct a new query with provided arguments
    pub fn new(args: A) -> Self {
        Query {
            args,
            _return: PhantomData,
        }
    }

    /// Returns a reference to the arguments of a query
    pub fn args(&self) -> &A {
        &self.args
    }

    /// Consumes query and returns the arguments
    pub fn into_args(self) -> A {
        self.args
    }
}

impl<Over, A, R, S, const ID: u8> Canon<S> for Query<Over, A, R, ID>
where
    A: Canon<S>,
    S: Store,
{
    fn write(&self, sink: &mut impl Sink<S>) -> Result<(), S::Error> {
        self.args.write(sink)
    }

    fn read(source: &mut impl Source<S>) -> Result<Self, S::Error> {
        Ok(Query {
            args: A::read(source)?,
            _return: PhantomData,
        })
    }

    fn encoded_len(&self) -> usize {
        self.args.encoded_len()
    }
}
