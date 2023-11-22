use std::error::Error;

use inkwell::AddressSpace;
use inkwell::values::{IntValue, BasicMetadataValueEnum, CallSiteValue, ArrayValue, BasicValueEnum};

use crate::codegen::codegen::{CodeGenable, Compiler};
use crate::ast::{self, Expr};
use crate::codegen::named_value_store::NamedValueStore;
use crate::types::character::CharValue;
use crate::types::traits::Puttable;


impl<'a, 'ctx> CodeGenable<'a,'ctx> for ast::Put
{
    unsafe fn codegen(self, compiler: &'a Compiler<'a, 'ctx>)
                -> Box<dyn inkwell::values::AnyValue<'ctx> + 'ctx> {
        
                    Box::new(compiler.print_string(self.message_to_print))
    }
}

impl<'a,'ctx> Compiler<'a,'ctx>
{
unsafe fn print_string(&'a self, message: Expr) -> CallSiteValue<'ctx> {

if let Expr::Char { value } = message.clone() {
    let genned_string = message.codegen(self);


    let string_array: ArrayValue<'ctx> =
        genned_string.as_any_value_enum().into_array_value();

    let char_value = CharValue::new(string_array);
    let bitc = char_value.get_pointer_to_printable_string(self);


 
    let res = self.builder.build_call(
        self.get_function("printf").unwrap(),
        &[BasicMetadataValueEnum::from(bitc)],
        "printf",
    );

    return res.unwrap();
}
else if let Expr::Variable { _type, name } = message.clone()
{
     

    let var_ptr = self.named_values.try_get(&name).unwrap();

    //let genned_string = message.codegen(self);



    //let string_array: ArrayValue<'ctx> =
    //    genned_string.as_any_value_enum().into_array_value();

    let arr_ptr = var_ptr.pointer;
    let bitc: BasicValueEnum<'ctx> = self
                    .builder
                    .build_bitcast(
                        arr_ptr,
                        self.context.i8_type().ptr_type(AddressSpace::default()),
                        "mybitcast",
                    )
                    .unwrap();



 
    let res = self.builder.build_call(
        self.get_function("printf").unwrap(),
        &[BasicMetadataValueEnum::from(bitc)],
        "printf",
    );

    return res.unwrap();


}
else {
    todo!("PUT doesn't support non strings yet! you passed in a {:?}",message);
}
}
}
