use inkwell::{types::{StructType, BasicTypeEnum}, values::ArrayValue};





///Represents a CHAR PL/1 value.
///A string is just an array of characters (which are i8 integers for ASCII)
#[derive(Debug)]
pub struct CharValue<'ctx>
{
    value: ArrayValue<'ctx>
}




pub fn get_character_type<'ctx>(ctx: &'ctx inkwell::context::Context, size_of_string: u32) -> StructType<'ctx>
{

        let mut field_types: Vec<BasicTypeEnum> = vec![];
        //let before_decimal_array = ctx.i8_type().array_type(BEFORE_DIGIT_COUNT);
        //let after_decimal_array = ctx.i8_type().array_type(AFTER_DIGIT_COUNT);
        //let is_negative_type = ctx.bool_type();
        //field_types.push(is_negative_type.as_basic_type_enum());
        //field_types.push(before_decimal_array.as_basic_type_enum());
        //field_types.push(after_decimal_array.as_basic_type_enum());
        
        let char_array = ctx.i8_type().array_type(size_of_string);

        let packed = false;
        ctx.struct_type(&field_types, packed)
}
