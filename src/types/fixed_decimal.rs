use inkwell::{types::{StructType, BasicType, BasicTypeEnum, ArrayType}, values::{StructValue, BasicValueEnum, ArrayValue, IntValue, FloatValue}};

use crate::codegen::codegen::Compiler;

const BEFORE_DIGIT_COUNT: u32 = 16;
const AFTER_DIGIT_COUNT: u32 = 15;

///Represents a Fixed PL/1 value.
///Currently can only represent a fixed decimal with
///16 digits before the decimal, 15 after.
#[derive(Debug)]
pub struct FixedValue<'ctx>
{
    value: StructValue<'ctx>
}

impl<'ctx> FixedValue<'ctx>
{
    pub fn new(value: StructValue<'ctx>) -> FixedValue<'ctx>
    {
        FixedValue { value }
    }
}

impl<'ctx> Into<StructValue<'ctx>> for FixedValue<'ctx>
{
    fn into(self) -> StructValue<'ctx> {
        self.value
    }
}

impl<'ctx> From<StructValue<'ctx>> for FixedValue<'ctx>
{
    fn from(value: StructValue<'ctx>) -> Self {
        FixedValue { value } 
    }
}








pub fn get_fixed_type<'ctx>(ctx: &'ctx inkwell::context::Context) -> StructType<'ctx>
{
        let mut field_types: Vec<BasicTypeEnum> = vec![];
        let before_decimal_array = ctx.i8_type().array_type(BEFORE_DIGIT_COUNT);
        let after_decimal_array = ctx.i8_type().array_type(AFTER_DIGIT_COUNT);
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
    let before_decimal_value = ctx.i8_type().array_type(BEFORE_DIGIT_COUNT).const_zero();
    let after_decimal_value = ctx.i8_type().array_type(AFTER_DIGIT_COUNT).const_zero();
    values.push(is_negative_value.into());
    values.push(before_decimal_value.into());
    values.push(after_decimal_value.into());

    _type.const_named_struct(&values)
    
}

///Coverts a f64 into a FixedValue
pub fn generate_fixed_decimal_code<'ctx>(ctx: &'ctx inkwell::context::Context, _type: StructType<'ctx>, value: f64) -> FixedValue<'ctx>
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
        before_decimal_digits.resize(BEFORE_DIGIT_COUNT as usize, ctx.i8_type().const_int(0, false));
    //now we gotta extract the number after the decimal as a positive integer
    let after_decimal_side = (value - before_decimal_side as f64) * 10_f64.powf(before_decimal_digits.len() as f64);
    let mut after_decimal_digits: Vec<IntValue> = convert_num_to_arr(after_decimal_side as i64)
        .iter()
        .map(
            |w| -> IntValue<'ctx> {
                ctx.i8_type().const_int(*w as u64, false)
            })
            .collect();
    
        after_decimal_digits.resize(AFTER_DIGIT_COUNT as usize, ctx.i8_type().const_int(0, false));

    
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

    FixedValue::new(_type.const_named_struct(&values))

    }


impl<'a, 'ctx> Compiler<'a, 'ctx>
{
    pub unsafe fn fixed_decimal_to_float(&self, fixed_value: FixedValue<'ctx>) -> FloatValue<'ctx>
    {

        return self.context.f64_type().const_zero();
        dbg!("CONVERTING TO FLOATIE!");
        let fixed_value_as_struct_value = fixed_value.value; 

        let pointer_to_structvalue = self.builder.build_alloca(fixed_value_as_struct_value.get_type(), "tmpalloca").unwrap();
        let sign_bit = self.builder.build_struct_gep(pointer_to_structvalue,0,"get_sign_bit").unwrap();

        let sign_bit_val = sign_bit.const_to_int(self.context.bool_type());


        let before_ptr = self.builder.build_struct_gep(pointer_to_structvalue,1,"get_before").unwrap();
        
        let before_arr = self.builder.build_load(before_ptr, "load_before_arr").unwrap().into_array_value();
        let zero_intval = self.context.i8_type().const_zero();
        let mut before_int_values: Vec<IntValue<'ctx>> = vec![zero_intval;BEFORE_DIGIT_COUNT as usize];

        for i in 0..BEFORE_DIGIT_COUNT as usize
        {
            let index_as_intval = self.context.i8_type().const_int(i as u64, false);
            before_int_values[i] = self
                .builder
                .build_gep(before_ptr, &[index_as_intval], "load_digit").unwrap()
                .const_to_int(self.context.i8_type());
        }
        
        let lhs = before_int_values[0];

        let mut result_floatval: FloatValue<'ctx> = lhs.const_unsigned_to_float(self.context.f64_type());

        for i in 1..BEFORE_DIGIT_COUNT as usize
        {
            let rhs = before_int_values[i].const_unsigned_to_float(self.context.f64_type());
            result_floatval = self.builder.build_float_add(result_floatval, rhs, "summer").unwrap();
        }
        
        //self.builder.build_gep(ptr, ordered_indexes, name)

        //let after_ptr = self.builder.build_struct_gep(res,2,"get_after").unwrap();
        //self.builder.build_struct_gep(res,2,"get_after");
        dbg!("FLOATVAL: {}",result_floatval);
        panic!("LOLO");
         result_floatval
        //return self.context.f64_type().const_float(float_const as f64);
    }











}

///Helper function
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
    }

    #[test]
    fn test_full_structy()
    {
        let ctx =inkwell::context::Context::create();

        let _type = get_fixed_type(&ctx);
        let myval = generate_fixed_decimal_code(&ctx, _type, 421.88888);

        dbg!(myval);
    }
}
