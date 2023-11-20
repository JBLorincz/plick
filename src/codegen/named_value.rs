use inkwell::values::PointerValue;

use crate::types::Type;


    #[derive(Debug, Clone)]
    pub struct NamedValue<'ctx> {
        pub name: String,
        pub _type: Type,
        pub pointer: PointerValue<'ctx>,
    }

    impl<'ctx> NamedValue<'ctx> {
        pub fn new(name: String, _type: Type, value: PointerValue<'ctx>) -> NamedValue<'ctx> {
            NamedValue { name, _type, pointer: value }
        }
    }
