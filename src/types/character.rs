use inkwell::{
    types::{ArrayType, BasicTypeEnum, StructType, BasicMetadataTypeEnum, FunctionType},
    values::{ArrayValue, IntValue, PointerValue, BasicValueEnum, BasicMetadataValueEnum}, AddressSpace, module::Linkage,
};

use crate::codegen::codegen::Compiler;

use super::{SIZE_OF_STRINGS, traits::Puttable};

///Represents a CHAR PL/1 value.
///A string is just an array of characters (which are i8 integers for ASCII)
#[derive(Debug)]
pub struct CharValue<'ctx> {
    pub value: ArrayValue<'ctx>,
}

impl<'ctx> CharValue<'ctx> {
    pub fn new(value: ArrayValue<'ctx>) -> CharValue<'ctx> {
        CharValue { value }
    }
}

impl<'ctx> Into<ArrayValue<'ctx>> for CharValue<'ctx> {
    fn into(self) -> ArrayValue<'ctx> {
        self.value
    }
}

pub fn get_character_type<'ctx>(
    ctx: &'ctx inkwell::context::Context,
    size_of_string: u32,
) -> ArrayType<'ctx> {

    //add 1 for the null terminator
    let char_array = ctx.i8_type().array_type(size_of_string + 1);

    char_array
}

///Converts a constant string into a charvalue
pub fn generate_character_code<'ctx>(
    ctx: &'ctx inkwell::context::Context,
    value: &str,
) -> CharValue<'ctx> {
    let string_with_terminator = value.to_string();
    let mut chars_as_numbers: Vec<IntValue> = vec![];
    let sign_extend = false;
    for char in string_with_terminator.chars() {
        let eight_bit_num: i8 = char as i8;
        let num: u64 = eight_bit_num as u64;
        chars_as_numbers.push(ctx.i8_type().const_int(num, sign_extend));
    }
    chars_as_numbers.push(ctx.i8_type().const_zero()); //terminator

    let value = ctx.i8_type().const_array(&chars_as_numbers[..]);

    CharValue { value }
}


pub fn generate_character_code_for_size<'ctx>(
    ctx: &'ctx inkwell::context::Context,
    value: &str,
    size: u32
) -> CharValue<'ctx> {
    let string_with_terminator = value.to_string();

    let mut chars_as_numbers: Vec<IntValue> = vec![];
    let sign_extend = false;


    for char in string_with_terminator.chars() {
        let eight_bit_num: i8 = char as i8;
        let num: u64 = eight_bit_num as u64;
        chars_as_numbers.push(ctx.i8_type().const_int(num, sign_extend));
    }


    chars_as_numbers.push(ctx.i8_type().const_zero()); //terminator
    let empty_character_count = size - string_with_terminator.chars().count() as u32;
    for i in 0..empty_character_count
    {
        chars_as_numbers.push(ctx.i8_type().const_int(1, false));
    }



    let value = ctx.i8_type().const_array(&chars_as_numbers[..]);

    CharValue { value }
}


impl<'a, 'ctx> Puttable<'a, 'ctx> for CharValue<'ctx>
{

        unsafe fn print_object(&self, compiler: &'a Compiler<'a, 'ctx>) 
        {
            let print_func = compiler.get_function("printf").unwrap();


            dbg!("WHAT");
            //let struc = self.value;
            //
            //let x = compiler.builder.build_bitcast(val, ty, name)

            //let name = "print_puttable";
            let mallocd_string = compiler.builder.build_malloc(self.value.get_type(), "storing_string").unwrap();
            compiler.builder.build_store(mallocd_string, self.value);
            
            let bitc: BasicValueEnum<'ctx> = compiler
                    .builder
                    .build_bitcast(
                        mallocd_string,
                        compiler.context.i8_type().ptr_type(AddressSpace::default()),
                        "mybitcast",
                    )
                    .unwrap();

            compiler.builder.build_call(print_func, &[bitc.into()], "printing_char_from_puttable").unwrap();


            compiler.builder.build_free(mallocd_string).unwrap();
        }
    fn get_pointer_to_printable_string(&self,compiler: &'a Compiler<'a,'ctx>) -> PointerValue<'ctx> {
        let string_array  = self.value;
                let const_string = self.value.get_string_constant().unwrap(); 
                let allocd_string = compiler.builder.build_global_string_ptr(const_string.to_str().unwrap(), "char_const").unwrap();
                
                let bitc: BasicValueEnum<'ctx> = compiler
                    .builder
                    .build_bitcast(
                        allocd_string,
                        compiler.context.i8_type().ptr_type(AddressSpace::default()),
                        "mybitcast",
                    )
                    .unwrap();

                bitc.into_pointer_value()

    }
}
