use inkwell::{types::{StructType, BasicType, BasicTypeEnum}, values::{StructValue, BasicValueEnum}};


pub enum Test
{
    Yes,
    No,
}

pub fn get_fixed_type<'ctx>(ctx: &'ctx inkwell::context::Context) -> StructType<'ctx>
{
        let mut field_types: Vec<BasicTypeEnum> = vec![];
        let before_decimal_array = ctx.i8_type().array_type(16);
        let after_decimal_array = ctx.i8_type().array_type(15);
        let is_negative_type = ctx.bool_type();
        field_types.push(is_negative_type.as_basic_type_enum());
        field_types.push(before_decimal_array.as_basic_type_enum());
        field_types.push(after_decimal_array.as_basic_type_enum());
        

        let packed = false;
        ctx.struct_type(&field_types, packed)
}




pub fn build_fixed_value<'ctx>(ctx: &'ctx inkwell::context::Context, _type: &'ctx StructType) -> StructValue<'ctx>
{
    let mut values: Vec<BasicValueEnum> = vec![];

    let is_negative_value = ctx.bool_type().const_int(0,false);
    let before_decimal_value = ctx.i8_type().array_type(16).const_zero();
    let after_decimal_value = ctx.i8_type().array_type(15).const_zero();
    values.push(is_negative_value.into());
    values.push(before_decimal_value.into());
    values.push(after_decimal_value.into());

    _type.const_named_struct(&values)
    
}




mod tests
{
    use inkwell::types::{BasicType, BasicTypeEnum};

    use super::{get_fixed_type, build_fixed_type};


   #[test]
    fn test_structy()
    {
        let ctx = inkwell::context::Context::create();

        let fixed_decimal_type = get_fixed_type(&ctx);

        let fixed_decimal = build_fixed_type(&ctx, &fixed_decimal_type);

        dbg!(fixed_decimal);

        panic!();
        
    }
}
