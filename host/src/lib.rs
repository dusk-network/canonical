#![feature(never_type)]

mod mem_store;
pub use mem_store::MemStore;

#[cfg(test)]
mod tests {
    #[test]
    fn counter() {
        let code = include_bytes!("../examples/counter/counter.wasm");
    }
}
