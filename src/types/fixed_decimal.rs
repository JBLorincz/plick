use inkwell::{types::{StructType, BasicType, BasicTypeEnum, ArrayType}, values::{StructValue, BasicValueEnum, ArrayValue, IntValue}};

use crate::codegen::codegen::Compiler;


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




pub fn create_empty_fixed<'ctx>(ctx: &'ctx inkwell::context::Context, _type: &'ctx StructType) -> StructValue<'ctx>
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

fn generate_fixed_decimal_code<'ctx>(ctx: &'ctx inkwell::context::Context, _type: StructType<'ctx>, value: f64) -> StructValue<'ctx>
    {


         let mut values: Vec<BasicValueEnum> = vec![];
        let negative_value_switch = 
            match value < 0.0
            {
                true => 1,
                false => 0
            };

    let is_negative_value = ctx.bool_type().const_int(negative_value_switch,false);

    //now we gotta extract the number before the decimal as a positive integer
    let before_decimal_side = value as u64; 
    let mut before_decimal_digits: Vec<IntValue> = convert_num_to_arr(before_decimal_side as i64)
        .iter()
        .map(
            |w| -> IntValue<'ctx> {
                ctx.i8_type().const_int(*w as u64, false)
            })
            .collect();
        before_decimal_digits.resize(16, ctx.i8_type().const_int(0, false));
    //now we gotta extract the number after the decimal as a positive integer
    let after_decimal_side = (value - before_decimal_side as f64) * 10_f64.powf(before_decimal_digits.len() as f64);
    let mut after_decimal_digits: Vec<IntValue> = convert_num_to_arr(after_decimal_side as i64)
        .iter()
        .map(
            |w| -> IntValue<'ctx> {
                ctx.i8_type().const_int(*w as u64, false)
            })
            .collect();
    
        after_decimal_digits.resize(15, ctx.i8_type().const_int(0, false));

    
    let before_decimal_value = 
        ctx
        .i8_type()
        .const_array(&before_decimal_digits[..]);
    let after_decimal_value = 
        ctx
        .i8_type()
        .const_array(&after_decimal_digits[..]);
    values.push(is_negative_value.into());
    values.push(before_decimal_value.into());
    values.push(after_decimal_value.into());

    _type.const_named_struct(&values)

    }

impl<'a, 'ctx> Compiler<'a, 'ctx>
{
//    fn generate_fixed_decimal_code(&'a self, value: f64) -> StructValue<'ctx>
//    {
//
//
//         let mut values: Vec<BasicValueEnum> = vec![];
//        let negative_value_switch = 
//            match value < 0.0
//            {
//                true => 1,
//                false => 0
//            };
//
//    let is_negative_value = self.context.bool_type().const_int(negative_value_switch,false);
//
//    //now we gotta extract the number before the decimal as a positive integer
//    let before_decimal_side = value as u64; 
//    let before_decimal_digits: Vec<IntValue> = convert_num_to_arr(before_decimal_side as i64)
//        .iter()
//        .map(
//            |w| -> IntValue<'ctx> {
//                self.context.i8_type().const_int(*w as u64, false)
//            })
//            .collect();
//    //now we gotta extract the number after the decimal as a positive integer
//    let after_decimal_side = (value - before_decimal_side as f64) * 10_f64.powf(before_decimal_digits.len() as f64);
//    let after_decimal_digits: Vec<IntValue> = convert_num_to_arr(after_decimal_side as i64)
//        .iter()
//        .map(
//            |w| -> IntValue<'ctx> {
//                self.context.i8_type().const_int(*w as u64, false)
//            })
//            .collect();
//    
//
//    
//    let before_decimal_value = self
//        .context
//        .i8_type()
//        .const_array(&before_decimal_digits[..]);
//    let after_decimal_value = self
//        .context
//        .i8_type()
//        .const_array(&after_decimal_digits[..]);
//    values.push(is_negative_value.into());
//    values.push(before_decimal_value.into());
//    values.push(after_decimal_value.into());
//
//    self.type_module.fixed_type.const_named_struct(&values)
//
//    }
}





//pub fn create_before_decimal<'a,'ctx>(c : &'a Compiler<'a,'ctx>) -> ArrayValue<'ctx>
//{
//
//    let before_decimal_value = c.context.i8_type().array_type(16).const_zero();
//    
//    c.builder.build_array_alloca(, size, name)
//    c.builder.build_gep(before_decimal_value., ordered_indexes, name)
//
//}
//pub fn build_fixed_val_int<'ctx>(ctx: &'ctx inkwell::context::Context, _type: &'ctx StructType) -> StructValue<'ctx>
//{
//    let mut values: Vec<BasicValueEnum> = vec![];
//
//    let is_negative_value = ctx.bool_type().const_int(0,false);
//    //let before_decimal_value = ctx.i8_type().array_type(16).const_zero();
//    //let after_decimal_value = ctx.i8_type().array_type(15).const_zero();
//    let (before_decimal_value, after_decimal_value) = convert_num_to_array_values(ctx, 42.0);
//    values.push(is_negative_value.into());
//    values.push(before_decimal_value.into());
//    values.push(after_decimal_value.into());
//
//    _type.const_named_struct(&values)
//    
//}




//internal functions



//fn convert_num_to_array_values<'ctx>(ctx: &'ctx inkwell::context::Context, num: f64)
//    ->(ArrayValue<'ctx>, ArrayValue<'ctx>)
//{
//
//    let before = from(num as i64);
//    let arrayval_vec = before.iter().map(
//    |what: &u8| 
//    {
//    ctx.i8_type().const_int(*what as u64, false)
//                     }).collect();
//    
//    let before_decimal_value = ctx.i8_type().array_type(16).;
//    //let before_decimal_value = arrayval_vec;
//    let after_decimal_value = ctx.i8_type().array_type(15).const_zero();
//    
//    return (before_decimal_value,after_decimal_value)
//}


fn convert_num_to_arr(value: i64) -> Vec<u8>
    {
        let mut value = value; 
        let mut before_decimal: Vec<u8> = vec![];

        if value < 0
        {
            value *= -1;
        }

        loop {
            let current_digit: u8 = (value % 10) as u8;
            before_decimal.push(current_digit);
            
            value = value / 10;

            if value == 0
            {
                break;
            }


        }
       
        before_decimal
    }
    
mod tests
{
    use inkwell::types::{BasicType, BasicTypeEnum};

    use crate::types::fixed_decimal::create_empty_fixed;

    use super::{get_fixed_type, generate_fixed_decimal_code};


   #[test]
    fn test_structy()
    {
        let ctx = inkwell::context::Context::create();

        let fixed_decimal_type = get_fixed_type(&ctx);

        let fixed_decimal = create_empty_fixed(&ctx, &fixed_decimal_type);

        dbg!(fixed_decimal);

        panic!();
        
    }

    #[test]
    fn test_full_structy()
    {
        let ctx =inkwell::context::Context::create();

        let _type = get_fixed_type(&ctx);
        let myval = generate_fixed_decimal_code(&ctx, _type, 421.88888);

        dbg!(myval);

        panic!("What?");
    }
}
