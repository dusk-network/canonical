use blake2b_simd::{Params, State as Blake2bState};

use crate::{IdBuilder, Ident};

/// A 32 byte Identifier based on the Blake2b hash algorithm
#[derive(Hash, PartialEq, Eq, Debug, Copy, Default, Clone)]
pub struct Id32([u8; 32]);

pub struct Id32Builder(Blake2bState);

impl Default for Id32Builder {
    fn default() -> Self {
        Id32Builder(Params::new().hash_length(32).to_state())
    }
}

impl IdBuilder<Id32> for Id32Builder {
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }

    fn fin(mut self) -> Id32 {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(self.0.finalize().as_ref());
        Id32(bytes)
    }
}

impl Ident for Id32 {
    type Builder = Id32Builder;
}

impl AsRef<[u8]> for Id32 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for Id32 {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
