use canon_derive::Canon;
use core::marker::PhantomData;

/// Represents the type of a transaction
#[derive(Debug, Clone, Canon)]
pub struct Transaction<Over, A, R, const ID: u8> {
    /// Arguments, in form of a tuple or single value
    args: A,
    /// The expected return type
    _return: PhantomData<(Over, R)>,
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
