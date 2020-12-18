use wasmi::{
    Error, Externals, FuncRef, ModuleImportResolver, RuntimeArgs, RuntimeValue,
    Signature, Trap,
};

const C_QUERY: usize = 0;

#[derive(Clone, Copy, Default)]
pub struct TestResolver;

impl Externals for TestResolver {
    fn invoke_index(
        &mut self,
        _index: usize,
        _args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        Ok(None)
    }
}

impl ModuleImportResolver for TestResolver {
    fn resolve_func(
        &self,
        field_name: &str,
        _signature: &Signature,
    ) -> Result<FuncRef, Error> {
        match field_name {
            "c_query" => Ok(wasmi::FuncInstance::alloc_host(
                wasmi::Signature::new(
                    &[
                        wasmi::ValueType::I32,
                        wasmi::ValueType::I32,
                        wasmi::ValueType::I32,
                    ][..],
                    None,
                ),
                C_QUERY,
            )),
            _ => panic!("unimplemented resolve {}", field_name),
        }
    }
}
