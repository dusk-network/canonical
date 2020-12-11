use core::marker::PhantomData;

/// Represents the type of a query
///
/// `Over` is the type that the query is expected to operate over.
#[derive(Debug, Clone, Canon)]
pub struct Query<Over, A, R, const ID: u8> {
    /// Arguments, in form of a tuple or single value
    args: A,
    /// The expected return type
    _return: PhantomData<(Over, R)>,
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
