use inkwell::values::{AnyValue, ArrayValue, FloatValue, PointerValue, StructValue};

use crate::codegen::codegen::Compiler;

use super::{
    character::CharValue, fixed_decimal::FixedValue, float_decimal::PLIFloatDecimalValue, Type,
};

pub trait Puttable<'a, 'ctx> {
    fn get_pointer_to_printable_string(
        &self,
        compiler: &'a Compiler<'a, 'ctx>,
    ) -> PointerValue<'ctx>;
    unsafe fn print_object(&self, compiler: &'a Compiler<'a, 'ctx>);
}

pub fn get_puttable_type<'a, 'ctx>(
    value: Box<dyn AnyValue<'ctx> + 'ctx>,
    _type: Type,
) -> Result<Box<dyn Puttable<'a, 'ctx> + 'ctx>, String> {
    let result: Box<dyn Puttable> = match _type {
        Type::FixedDecimal => {
            let struc: StructValue<'ctx> = value.as_any_value_enum().into_struct_value();
            let fd: FixedValue<'ctx> = FixedValue::new(struc);
            Box::new(fd)
        }
        Type::Float => {
            let struc: StructValue<'ctx> = value.as_any_value_enum().into_struct_value();
            let fd: PLIFloatDecimalValue<'ctx> = PLIFloatDecimalValue::new(struc);
            Box::new(fd)
        }
        Type::Char(_size) => {
            let char_array: ArrayValue<'ctx> = value.as_any_value_enum().into_array_value();
            let char_value: CharValue<'ctx> = CharValue::new(char_array);
            Box::new(char_value)
        }
        other => panic!("Cant make puttable type {}", other),
    };

    Ok(result)
}

pub trait Mathable<'a, 'ctx> {
    fn convert_to_float(&self, compiler: &'a Compiler<'a, 'ctx>) -> FloatValue<'ctx>;
    //fn convert_from_float(float_value: &FloatValue<'ctx>, compiler: &'a Compiler<'a,'ctx>) -> Box<dyn Mathable<'a,'ctx>>;
}

pub trait MathableFactory<'a, 'ctx, T>
where
    T: Mathable<'a, 'ctx>,
{
    unsafe fn create_mathable(float: &FloatValue<'ctx>, compiler: &Compiler<'a, 'ctx>) -> Box<T>;
}

impl<'a, 'ctx> MathableFactory<'a, 'ctx, FixedValue<'ctx>> for FixedValue<'ctx> {
    unsafe fn create_mathable(
        float: &FloatValue<'ctx>,
        compiler: &Compiler<'a, 'ctx>,
    ) -> Box<FixedValue<'ctx>> {
        Box::new(compiler.float_value_to_fixed_decimal(float.clone()))
    }
}

pub fn get_mathable_type<'a, 'ctx>(
    value: Box<dyn AnyValue<'ctx> + 'ctx>,
    _type: Type,
) -> Result<Box<dyn Mathable<'a, 'ctx> + 'ctx>, String> {
    match _type {
        Type::FixedDecimal => {
            let struct_value: StructValue<'ctx> = value.as_any_value_enum().into_struct_value();

            let fixed_value = FixedValue::new(struct_value);
            Ok(Box::new(fixed_value))
        }
        Type::Float => {
            let struct_value: StructValue<'ctx> = value.as_any_value_enum().into_struct_value();

            let pli_float_value = PLIFloatDecimalValue::new(struct_value);
            Ok(Box::new(pli_float_value))
        }
        other => Err(format!("Type {:#?} is not mathable", other)),
    }
}
