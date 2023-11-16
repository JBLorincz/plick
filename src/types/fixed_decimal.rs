use inkwell::{
    types::{ArrayType, BasicType, BasicTypeEnum, StructType},
    values::{ArrayValue, BasicValueEnum, FloatValue, IntValue, StructValue, PointerValue, AnyValue}, AddressSpace,
};

use crate::codegen::{codegen::Compiler, utils::{self, get_nth_digit_of_a_float, print_int_value, build_pow, print_float_value}};

use super::{Puttable, traits::Mathable};

const BEFORE_DIGIT_COUNT: u32 = 16;
const AFTER_DIGIT_COUNT: u32 = 15;

///Represents a Fixed PL/1 value.
///Currently can only represent a fixed decimal with
///16 digits before the decimal, 15 after.
#[derive(Debug)]
pub struct FixedValue<'ctx> {
    value: StructValue<'ctx>,
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
    fn get_pointer_to_printable_string(&self, compiler: &'a Compiler<'a, 'ctx>) -> PointerValue<'ctx> {

         let mut fd_to_float_converter = FixedDecimalToFloatBuilder::new(compiler,&self.value);
         let const_int_zero = compiler.context.i8_type().const_zero();
        fd_to_float_converter.alloca_struct_value();

        fd_to_float_converter.get_sign_bit_value();
        
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

        before_int_values.splice(0..0, [compiler.context.i8_type().const_int(40,false)]);
        
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

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    pub unsafe fn fixed_decimal_to_float(&self, fixed_value: &FixedValue<'ctx>) -> FloatValue<'ctx> {
        dbg!("Converting fixed value {} into a decimal!", fixed_value);
        self.print_const_string("im now printing the same fixed value but in the float conv:\n\0");
        self.print_puttable(fixed_value);
        let fixed_value_as_struct_value: StructValue<'ctx> = fixed_value.value;

        let mut fd_to_float_converter = FixedDecimalToFloatBuilder::new(self,&fixed_value_as_struct_value);

        fd_to_float_converter.alloca_struct_value();

        fd_to_float_converter.get_sign_bit_value();
        
        let ptr_to_before_array = fd_to_float_converter.get_before_ptr();

        dbg!(ptr_to_before_array);
        let before_arr = fd_to_float_converter.get_before_array();
        dbg!(before_arr);

        let zero_intval = self.context.i8_type().const_zero();
        let mut before_int_values: Vec<IntValue<'ctx>> =
            vec![zero_intval; BEFORE_DIGIT_COUNT as usize];

        for i in 0..BEFORE_DIGIT_COUNT as usize {
            self.print_const_string("Digit: ");
            let current_digit_index = self.context.i8_type().const_int(i as u64, false);
        
            let digit_int_val = fd_to_float_converter.load_digit_from_digit_array(current_digit_index, ptr_to_before_array);
            print_int_value(self,digit_int_val);  

            //now we take the array value, build a GEP for the inner array
            before_int_values[i] = digit_int_val;
        }

        let result_floatval = fd_to_float_converter.sum_up_before_digits_into_a_float(before_int_values);

        result_floatval
    }


    pub unsafe fn float_value_to_fixed_decimal(&'a self, float_value: FloatValue<'ctx>) -> FixedValue<'ctx>
    {
        let zero_intval = self.context.i8_type().const_zero();
        let fixed_value: StructValue<'ctx> = create_empty_fixed(self.context, &self.type_module.fixed_type);
        let allocd_fd: PointerValue<'ctx> = self.builder.build_alloca(fixed_value.get_type(), "allocate_fd_for_fixed_conv").unwrap();
        self.builder.build_store(allocd_fd, fixed_value).unwrap();
        let allocd_float = self.builder.build_alloca(float_value.get_type(), "allocate_float_for_fixed_conv").unwrap();


        let rhs: FloatValue<'ctx> = float_value.get_type().const_zero();


            let conditional = self
            .builder
            .build_float_compare(
                inkwell::FloatPredicate::OLT,
                float_value,
                rhs,
                "is_float_neg")
            .unwrap();

            //start of sign bit loading
            let current_func = utils::get_current_function(self);
            let then_block = self.context.append_basic_block(current_func, "then_block");
            let else_block = self.context.append_basic_block(current_func, "else_block");
            let cont_block = self.context.append_basic_block(current_func, "cont_block");
                self.
                builder.
                build_conditional_branch(conditional, then_block, else_block)
                .expect("This should be fine!");

            self.builder.position_at_end(then_block);
            
            let sign_ptr = self.builder.build_struct_gep(allocd_fd, 0, "get_sign_bit_ptr").unwrap();
            self.builder.build_store(sign_ptr, self.context.i8_type().const_int(1, false)).unwrap();

                      
            
            self.builder.position_at_end(cont_block);
            //end of sign bit loading

            //start of calculating fv digits
            let bdcl = self.context.append_basic_block(current_func, "before_digits_calculation_loop");
            self.builder.position_at_end(bdcl);

            //counter 

            let before_array_ptr = self.builder.build_struct_gep(allocd_fd, 1, "get_before_array_ptr").unwrap();
            for i in 0..16
            {
                let index_as_intval = self.context.i8_type().const_int(i, false);
                let digit_to_load_up = get_nth_digit_of_a_float(self, &float_value, index_as_intval);

                let current_digit_ptr = self.builder.build_gep(before_array_ptr,&[zero_intval,index_as_intval],"getdigiforconv")
                    .unwrap();

                self.builder.build_store(current_digit_ptr, digit_to_load_up);
            }
            

            let after_before_loop = self.context.append_basic_block(current_func, "after_digits_loop");
            self.builder.position_at_end(after_before_loop);
            //end of calculating fv digits

        
        let fd_struct: StructValue<'ctx> = self.builder.build_load(allocd_fd, "loading_final_fd").unwrap().into_struct_value(); 
        FixedValue::new(fd_struct)
    }
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
            //panic!("{}",my_float);
            let rhs = f64_type.const_float(my_float);
            self.compiler.print_const_string(" MY RHS IS: ");
            print_float_value(self.compiler, rhs);
            self.compiler.print_const_string("  ");
            let digit_log = build_pow(self.compiler, ten_float, rhs);

            self.compiler.print_const_string("digit_log is:");
            print_float_value(self.compiler,digit_log);

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
        //let lhs = before_int_values[0];

        //let mut result_floatval: FloatValue<'ctx> = self.compiler
        //    .builder
        //    .build_unsigned_int_to_float(
        //        before_int_values[0],
        //        self.compiler.context.f64_type(),
        //        "digAsFloat",
        //    )
        //    .unwrap();

        //for i in 1..BEFORE_DIGIT_COUNT as usize {
        //    let float = self
        //        .compiler
        //        .builder
        //        .build_unsigned_int_to_float(
        //            before_int_values[i],
        //            self.compiler.context.f64_type(),
        //            "digAsFloat",
        //        )
        //        .unwrap();

        //    result_floatval = self
        //        .compiler
        //        .builder
        //        .build_float_add(result_floatval, float, "summer")
        //        .unwrap();
        //}
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
