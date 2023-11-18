use inkwell::values::{FloatValue, AnyValue, StructValue, PointerValue};

use crate::codegen::codegen::Compiler;

use super::{Type, fixed_decimal::FixedValue};

pub trait Puttable<'a, 'ctx>
{
    fn get_pointer_to_printable_string(&self, compiler: &'a Compiler<'a, 'ctx>) -> PointerValue<'ctx>;
}



pub trait Mathable<'a, 'ctx>
{
    fn convert_to_float(&self, compiler: &'a Compiler<'a,'ctx>) -> FloatValue<'ctx>;
    //fn convert_from_float(float_value: &FloatValue<'ctx>, compiler: &'a Compiler<'a,'ctx>);
}


pub fn get_mathable_type<'a,'ctx>(value: Box<dyn AnyValue<'ctx> +'ctx>, _type: Type) -> Result<Box<dyn Mathable<'a,'ctx> +'ctx>, String>
{
    let struct_value: StructValue<'ctx> = value
        .as_any_value_enum()
        .into_struct_value();

    let fixed_value = FixedValue::new(struct_value);
    Ok(Box::new(fixed_value))
}
