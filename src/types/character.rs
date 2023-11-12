use inkwell::{types::{StructType, BasicTypeEnum, ArrayType}, values::{ArrayValue, IntValue}};

use super::SIZE_OF_STRINGS;





///Represents a CHAR PL/1 value.
///A string is just an array of characters (which are i8 integers for ASCII)
#[derive(Debug)]
pub struct CharValue<'ctx>
{
    value: ArrayValue<'ctx>
}

impl<'ctx> CharValue<'ctx>
{
    pub fn new(value: ArrayValue<'ctx>) -> CharValue<'ctx>
    {
        CharValue { value }
    }
}

impl<'ctx> Into<ArrayValue<'ctx>> for CharValue<'ctx>
{
    fn into(self) -> ArrayValue<'ctx> {
        self.value
    }
}






pub fn get_character_type<'ctx>(ctx: &'ctx inkwell::context::Context, size_of_string: u32) -> ArrayType<'ctx>
{

        let mut field_types: Vec<BasicTypeEnum> = vec![];
        
        //add 1 for the null terminator
        let char_array = ctx.i8_type().array_type(size_of_string + 1);

        let packed = false;
        char_array
}


///Coverts a f64 into a FixedValue
pub fn generate_character_code<'ctx>(ctx: &'ctx inkwell::context::Context, value: &str) -> CharValue<'ctx>
{

    let string_with_terminator = value.to_string();
    let mut chars_as_numbers: Vec<IntValue> = vec![];
    let sign_extend = false;
    for char in string_with_terminator.chars()
    {
        let eight_bit_num : i8 = char as i8;
        let num : u64 = eight_bit_num as u64;
        chars_as_numbers.push(ctx.i8_type().const_int(num, sign_extend));
    }
        chars_as_numbers.push(ctx.i8_type().const_zero());//terminator
    
    let value = ctx.i8_type().const_array(&chars_as_numbers[..]);

    CharValue { value  }
}
