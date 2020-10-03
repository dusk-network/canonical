use blake2b_simd::Params;

/// A 32 bit Identifier based on the Blake2b hash algorithm
#[derive(Hash, PartialEq, Eq, Debug, Copy, Default, Clone)]
pub struct Id32([u8; 32]);

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

impl From<&[u8]> for Id32 {
    fn from(bytes: &[u8]) -> Self {
        let mut ident = Id32::default();
        let hash = Params::new()
            .hash_length(32)
            .to_state()
            .update(bytes)
            .finalize();
        ident.as_mut()[..].copy_from_slice(hash.as_ref());
        ident
    }
}
