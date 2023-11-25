use inkwell::{
    types::{ArrayType, BasicType, BasicTypeEnum, StructType, FunctionType, BasicMetadataTypeEnum},
    values::{ArrayValue, BasicValueEnum, FloatValue, IntValue, StructValue, PointerValue, AnyValue, BasicMetadataValueEnum, BasicValue}, AddressSpace, module::Linkage,
};

use crate::codegen::{codegen::Compiler, utils::{self, get_nth_digit_of_a_float, print_int_value, build_pow, print_float_value, get_current_function}};

use super::{traits::{Puttable, Mathable}, Type};

mod float_to_fixed;


const BEFORE_DIGIT_COUNT: u32 = 16;
const AFTER_DIGIT_COUNT: u32 = 15;

const PLUS_ASCII_CODE: u64 = 43;
const MINUS_ASCII_CODE: u64 = 45;
///Represents a Fixed PL/1 value.
///Currently can only represent a fixed decimal with
///16 digits before the decimal, 15 after.
#[derive(Debug)]
pub struct FixedValue<'ctx> {
    pub value: StructValue<'ctx>,
}

impl<'ctx> FixedValue<'ctx> {
    pub fn new(value: StructValue<'ctx>) -> FixedValue<'ctx> {
        FixedValue { value }
    }
}

impl<'ctx> Into<StructValue<'ctx>> for FixedValue<'ctx> {
    fn into(self) -> StructValue<'ctx> {
        self.value
    }
}

impl<'ctx> From<StructValue<'ctx>> for FixedValue<'ctx> {
    fn from(value: StructValue<'ctx>) -> Self {
        let fixed_value = FixedValue { value };



        fixed_value
    }
}


impl<'a,'ctx> Puttable<'a,'ctx> for FixedValue<'ctx>
{
        unsafe fn print_object(&self, compiler: &'a Compiler<'a, 'ctx>) 
        {
            let print_func = compiler.get_function("print_fixed_decimal").unwrap();

            let struc = self.value;

            let name = "print_puttable";

            compiler.builder.build_call(print_func, &[struc.into()], name).unwrap();
    }


    fn get_pointer_to_printable_string(&self, compiler: &'a Compiler<'a, 'ctx>) -> PointerValue<'ctx> {

         let const_int_zero = compiler.context.i8_type().const_zero();

         let mut fd_to_float_converter = FixedDecimalToFloatBuilder::new(compiler,&self.value);

         fd_to_float_converter.alloca_struct_value();
        let sign_bit_value = fd_to_float_converter.get_sign_bit_ascii_value();
        let ptr_to_before_array = fd_to_float_converter.get_before_ptr();

        dbg!(ptr_to_before_array);
        let before_arr = fd_to_float_converter.get_before_array();
        dbg!(before_arr);

        let zero_intval = compiler.context.i8_type().const_zero();
        let mut before_int_values: Vec<IntValue<'ctx>> =
            vec![zero_intval; BEFORE_DIGIT_COUNT as usize];

        //before_int_values.push(compiler.context.i8_type().const_int(40,false));
        for i in 0..BEFORE_DIGIT_COUNT as usize {
            let current_digit_index = compiler.context.i8_type().const_int(i as u64, false);

            unsafe
            {
            let digit_int_val = fd_to_float_converter.load_digit_from_digit_array(current_digit_index, ptr_to_before_array);
            

            //now we take the array value, build a GEP for the inner array
            //BECAUSE WE WANT ASCII WE ADD 48 TO EVERY DIGIT
            let ascii_ofset =  compiler.context.i8_type().const_int(48, false);
            before_int_values[i] =  compiler
                .builder
                .build_int_add(digit_int_val, ascii_ofset, "ascioffset")
                .unwrap();
            }
        }
        //null terminator
        before_int_values.push(compiler.context.i8_type().const_int(41,false));
        before_int_values.push(compiler.context.i8_type().const_zero());

        before_int_values.splice(0..0, [sign_bit_value,compiler.context.i8_type().const_int(40,false)]);
        
        //HARD DECK
        let _typ = compiler.context.i8_type().array_type(before_int_values.len() as u32);
        let aloca = compiler.builder.build_alloca(_typ, "fd_before_as_string").unwrap();

        //let aloca = compiler.builder.build_alloca(compiler.context.i8_type().array_type(3), "fd_before_as_string").unwrap();
        compiler
            .builder
            .build_store(aloca, 
                         _typ.const_zero())
            .unwrap();


        unsafe {


            for (index,intval) in before_int_values.iter().enumerate()
            {
                        let current_index_as_intval = compiler.context.i8_type().const_int(index as u64, false);
                        let elemptr = compiler
                .builder.build_gep(aloca, &[const_int_zero,current_index_as_intval], "lol")
                .unwrap();

                compiler.builder.build_store(elemptr,*intval).expect("Intval wasnt an intval!");
            }

        }




        let aloca = compiler.builder.build_bitcast(
                        aloca,
                        compiler.context.i8_type().ptr_type(AddressSpace::default()),
                        "mybitcast",
                    );
        aloca.unwrap().into_pointer_value()

       }
}

impl<'a,'ctx> Mathable<'a,'ctx> for FixedValue<'ctx>
{
    fn convert_to_float(&self, compiler: &'a Compiler<'a,'ctx>) -> FloatValue<'ctx> {
        unsafe {
        compiler.fixed_decimal_to_float(self)
        }
    }
}








pub fn get_fixed_type<'ctx>(ctx: &'ctx inkwell::context::Context) -> StructType<'ctx> {
    let mut field_types: Vec<BasicTypeEnum> = vec![];
    //previously before decimal array
    let left_of_decimal_array = ctx.i8_type().array_type(BEFORE_DIGIT_COUNT);
    //previously after decimal array
    let right_of_decimal_array = ctx.i8_type().array_type(AFTER_DIGIT_COUNT);
    let is_negative_type = ctx.bool_type();
    field_types.push(is_negative_type.as_basic_type_enum());
    field_types.push(left_of_decimal_array.as_basic_type_enum());
    field_types.push(right_of_decimal_array.as_basic_type_enum());

    let packed = false;
    ctx.struct_type(&field_types, packed)
}

pub fn create_empty_fixed<'a,'ctx>(
    ctx: &'ctx inkwell::context::Context,
    _type: &'a StructType<'ctx>,
) -> StructValue<'ctx> {
    let mut values: Vec<BasicValueEnum> = vec![];

    let is_negative_value = ctx.bool_type().const_int(0, false);
    let left_of_decimal_value = ctx.i8_type().array_type(BEFORE_DIGIT_COUNT).const_zero();
    let right_of_decimal_value = ctx.i8_type().array_type(AFTER_DIGIT_COUNT).const_zero();
    values.push(is_negative_value.into());
    values.push(left_of_decimal_value.into());
    values.push(right_of_decimal_value.into());

    let struc = _type.const_named_struct(&values);

    struc
}

///Coverts a f64 into a FixedValue
pub fn generate_fixed_decimal_code<'ctx>(
    ctx: &'ctx inkwell::context::Context,
    _type: StructType<'ctx>,
    value: f64,
) -> FixedValue<'ctx> {
    let mut values: Vec<BasicValueEnum> = vec![];
    let negative_value_switch = match value < 0.0 {
        true => 1,
        false => 0,
    };

    let is_negative_value = ctx.bool_type().const_int(negative_value_switch, false);

    //now we gotta extract the number before the decimal as a positive integer
    let before_decimal_side = value as u64;
    let mut before_decimal_digits: Vec<IntValue> = convert_num_to_arr(before_decimal_side as i64)
        .iter()
        .map(|w| -> IntValue<'ctx> { ctx.i8_type().const_int(*w as u64, false) })
        .collect();


    before_decimal_digits.resize(
        BEFORE_DIGIT_COUNT as usize,
        ctx.i8_type().const_int(0, false),
    );
    //now we gotta extract the number after the decimal as a positive integer
    let after_decimal_side =
        (value - before_decimal_side as f64) * 10_f64.powf(before_decimal_digits.len() as f64);
    let mut after_decimal_digits: Vec<IntValue> = convert_num_to_arr(after_decimal_side as i64)
        .iter()
        .map(|w| -> IntValue<'ctx> { ctx.i8_type().const_int(*w as u64, false) })
        .collect();

    after_decimal_digits.resize(
        AFTER_DIGIT_COUNT as usize,
        ctx.i8_type().const_int(0, false),
    );

    let before_decimal_value = ctx.i8_type().const_array(&before_decimal_digits[..]);
    let after_decimal_value = ctx.i8_type().const_array(&after_decimal_digits[..]);
    values.push(is_negative_value.into());
    values.push(before_decimal_value.into());
    values.push(after_decimal_value.into());

    let new_fixed_value = FixedValue::new(_type.const_named_struct(&values));

    new_fixed_value
}



struct FixedDecimalToFloatBuilder<'a, 'b, 'ctx>
{
    compiler: &'a Compiler<'a,'ctx>,
    fd: &'b StructValue<'ctx>,
    pointer_to_struct_value: Option<PointerValue<'ctx>>
}

impl <'a, 'b, 'ctx> FixedDecimalToFloatBuilder<'a,'b,'ctx> {

    fn new(compiler: &'a Compiler<'a,'ctx>, fd: &'b StructValue<'ctx>) -> Self
    {
        FixedDecimalToFloatBuilder
        {
            compiler,
            fd,
            pointer_to_struct_value: None,
        }

    }
   fn alloca_struct_value(&mut self) -> PointerValue<'ctx>
    {
        let pointer_to_struct_value: PointerValue<'ctx> = self.compiler.builder
            .build_alloca(self.fd.get_type(), "tmpalloca")
            .unwrap();
        let my_fd: StructValue<'ctx> = self.fd.clone();
        self.compiler.builder.build_store(pointer_to_struct_value,my_fd).unwrap();
        self.pointer_to_struct_value = Some(pointer_to_struct_value);

        pointer_to_struct_value
    }
    fn get_sign_bit_ascii_value(&self) -> IntValue<'ctx>
   {
       let sign_bit_value = self.get_sign_bit_value();
       let builder = self.compiler.builder;
       let function = get_current_function(self.compiler);

        let then_block = self.compiler.context.append_basic_block(function, "if_negative");
        let else_block = self.compiler.context.append_basic_block(function, "if_positive");
        let merge_block = self.compiler.context.append_basic_block(function, "merge");

        let rhs = self.compiler.context.bool_type().const_int(1, false);
        let comparison =
            builder
            .build_int_compare(inkwell::IntPredicate::EQ, sign_bit_value, rhs, "is_negative")
            .unwrap();


            builder.build_conditional_branch(comparison, then_block, else_block).unwrap(); 

            builder.position_at_end(then_block);
            let negative_value = self.compiler.context.i8_type().const_int(MINUS_ASCII_CODE, false);
            builder.build_unconditional_branch(merge_block).unwrap();

            builder.position_at_end(else_block);
            let positive_value = self.compiler.context.i8_type().const_int(PLUS_ASCII_CODE, false);
            builder.build_unconditional_branch(merge_block).unwrap();

            builder.position_at_end(merge_block);

            let x = 
                builder.build_phi(self.compiler.context.i8_type(), "sign_character").unwrap();

                x.add_incoming(&[(&negative_value.as_basic_value_enum(), then_block)]);
                x.add_incoming(&[(&positive_value.as_basic_value_enum(), else_block)]);

                x.as_basic_value().into_int_value()
   }
   fn get_sign_bit_value(&self) -> IntValue<'ctx>
   {
        let sign_bit_ptr = self.compiler
            .builder
            .build_struct_gep(self.pointer_to_struct_value.unwrap(), 0, "get_sign_bit")
            .unwrap();

        let sign_bit_val = self.compiler
            .builder
            .build_load(sign_bit_ptr, "sign_bit")
            .unwrap()
            .into_int_value();

        sign_bit_val

   }
   fn get_before_ptr(&self) -> PointerValue<'ctx>
    {
            self.compiler.builder
            .build_struct_gep(self.pointer_to_struct_value.unwrap(), 1, "get_before")
            .unwrap()
    }
   fn get_before_array(&self) -> ArrayValue<'ctx>
    {       
        let before_ptr = self.get_before_ptr();    

        self.compiler
            .builder
            .build_load(before_ptr, "load_before_arr")
            .unwrap()
            .into_array_value()
    }

   unsafe fn load_digit_from_digit_array(&self, index: IntValue<'ctx>, ptr_to_array: PointerValue<'ctx>) ->
       IntValue<'ctx>
   {
       
        let zero_intval = self.compiler.context.i8_type().const_zero();
        let pointer_to_digit_array_value = self.compiler
                .builder
                .build_gep(ptr_to_array, &[zero_intval, index], "load_digit")
                .unwrap();


            let digit_int_val = self.compiler
                .builder
                .build_load(pointer_to_digit_array_value, "diggit")
                .unwrap()
                .into_int_value();

            digit_int_val
   }

    unsafe fn sum_up_before_digits_into_a_float(&self, before_int_values: Vec<IntValue<'ctx>>) -> FloatValue<'ctx>
    {
        let f64_type = self.compiler.context.f64_type();
        let result_float: FloatValue<'ctx>;
        let result_float_alloca = self
            .compiler
            .builder
            .build_alloca(self.compiler.context.f64_type(), "store_result")
            .unwrap();

        self.compiler.builder.build_store(result_float_alloca, f64_type.const_zero()).unwrap();

        let ten_float = self.compiler.context.f64_type().const_float(10.0);
        for (index,digit_value) in before_int_values.iter().enumerate()
        {
            let my_float = index as f64;

            let rhs = f64_type.const_float(my_float);
            let digit_log = build_pow(self.compiler, ten_float, rhs);

            let digit_scoped: IntValue<'ctx> = *digit_value;

            let digit_value_as_float = self.
                compiler.
                builder.
                build_unsigned_int_to_float(digit_scoped, f64_type, "lol")
                .unwrap();




            let digit_pure_value = self.compiler.builder.build_float_mul(digit_value_as_float, digit_log, "calc_digit_value")
                .unwrap();
            
            let old_value = self
                .compiler
                .builder
                .build_load(result_float_alloca, "load_old_val")
                .unwrap()
                .into_float_value();
            let new_value = self.compiler.builder.build_float_add(old_value, digit_pure_value, "lol").unwrap();

            self.compiler.builder.build_store(result_float_alloca, new_value).unwrap();
        }

        result_float = self.compiler.builder.build_load(result_float_alloca,"load_result").unwrap().into_float_value();

        result_float

    }




}

///Helper function
fn convert_num_to_arr(value: i64) -> Vec<u8> {
    let mut value = value;
    let mut before_decimal: Vec<u8> = vec![];

    if value < 0 {
        value *= -1;
    }

    loop {
        let current_digit: u8 = (value % 10) as u8;
        before_decimal.push(current_digit);

        value = value / 10;

        if value == 0 {
            break;
        }
    }

    before_decimal
}


pub fn add_fd_print_function<'a,'ctx>(compiler: &mut Compiler<'a,'ctx>)
{
    let mut param_types: Vec<BasicMetadataTypeEnum> = vec![];

    let fd_type = compiler.type_module.fixed_type;

    param_types.push(fd_type.into());

    let ty: FunctionType<'ctx> = compiler.context.void_type().fn_type(&param_types[..], false);

    let print_fd_func = compiler.module.add_function("print_fixed_decimal", ty, Some(Linkage::Internal));

    let current_bb = compiler.builder.get_insert_block().unwrap();

    let func_bb = compiler.context.append_basic_block(print_fd_func, "entry");

    compiler.builder.position_at_end(func_bb);

    let fixedvalue_param = func_bb.get_parent().unwrap().get_first_param().unwrap();

    let alloca_value = compiler.builder.build_alloca(fixedvalue_param.get_type(), "num_to_print").unwrap();

    compiler.builder.build_store(alloca_value, fixedvalue_param.into_struct_value()).unwrap();

    let mut args: Vec<BasicMetadataValueEnum> = vec![];
    let fd = FixedValue::from(fixedvalue_param.into_struct_value());
    let ptr = fd.get_pointer_to_printable_string(compiler);

    args.push(ptr.into());

    let func = compiler.module.get_function("printf").unwrap();
    compiler.builder.build_call(func, &args[..], "lol").unwrap();
    compiler.builder.build_return(None).unwrap();


    compiler.builder.position_at_end(current_bb);

}






mod tests {
    use inkwell::types::{BasicType, BasicTypeEnum};

    use crate::types::fixed_decimal::create_empty_fixed;

    use super::{generate_fixed_decimal_code, get_fixed_type};

    #[test]
    fn test_structy() {
        let ctx = inkwell::context::Context::create();

        let fixed_decimal_type = get_fixed_type(&ctx);

        let fixed_decimal = create_empty_fixed(&ctx, &fixed_decimal_type);

        dbg!(fixed_decimal);
    }

    #[test]
    fn test_full_structy() {
        let ctx = inkwell::context::Context::create();

        let _type = get_fixed_type(&ctx);
        let myval = generate_fixed_decimal_code(&ctx, _type, 421.88888);

        dbg!(myval);
    }
}
