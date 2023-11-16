use inkwell::values::{FunctionValue, IntValue, FloatValue, BasicMetadataValueEnum, PointerValue};

use super::codegen::Compiler;

/// A module that holds code generation utilities
/// that are reused across the application.




pub fn get_current_function<'a,'ctx>(compiler: &'a Compiler<'a,'ctx>) -> FunctionValue<'ctx>
{
    let current_func = compiler
                .builder
                .get_insert_block()
                .unwrap()
                .get_parent()
                .unwrap();

    current_func
}


pub fn get_nth_digit_of_a_float<'a,'ctx>(compiler: &'a Compiler<'a,'ctx>, float: &FloatValue<'ctx>, index: IntValue<'ctx>) -> IntValue<'ctx>
{
    let ten = compiler.context.i64_type().const_int(10, false);
    let ten_float = compiler.context.f64_type().const_float(10.00);

    let float_as_int = compiler.builder.build_float_to_unsigned_int(float.clone(), compiler.context.i64_type(), "float_as_int")
        .unwrap();
let index_as_float = compiler.builder.build_unsigned_int_to_float(index, compiler.context.f64_type(), "index_as_float")
        .unwrap();



    let divisor = build_pow(compiler,ten_float,index_as_float);
    //let divisor = compiler.context.f64_type().const_float(10.0);
    let divisor_as_int = compiler.builder.build_float_to_unsigned_int(divisor, compiler.context.i64_type(), "div_as_int")
        .unwrap();
    //digit = (number // divisor) % 10
    let num_to_operate_on = compiler.builder.build_int_unsigned_div(float_as_int, divisor_as_int, "find num to get digit off of")
        .unwrap();

    let digit = compiler.builder.build_int_unsigned_rem(num_to_operate_on, ten, "calc_digit");
    digit.unwrap()

}


pub fn build_pow<'a,'ctx>(compiler: &'a Compiler<'a,'ctx>, lhs: FloatValue<'ctx>, rhs: FloatValue<'ctx>) -> FloatValue<'ctx>
{
    let pow_name = "pow";
    let func = compiler.module.get_function(pow_name).unwrap();
    let args = &[BasicMetadataValueEnum::from(lhs), BasicMetadataValueEnum::from(rhs)];
    let res = compiler.builder.build_call(func,args,pow_name).unwrap();

    let result: FloatValue<'ctx> = res.try_as_basic_value().left().unwrap().into_float_value();
    result
}

pub fn print_float_value<'a,'ctx>(compiler: &'a Compiler<'a,'ctx>, float: FloatValue<'ctx>)
{
    let func_name = "printf";
    let template_string: PointerValue<'ctx> = compiler.builder.build_global_string_ptr("%lf", "glob_float_print").unwrap().as_pointer_value();
    let func = compiler.module.get_function(func_name).unwrap();
    let args = &[BasicMetadataValueEnum::from(template_string), BasicMetadataValueEnum::from(float)];
    let res = compiler.builder.build_call(func,args,func_name).unwrap();

}
pub fn print_int_value<'a,'ctx>(compiler: &'a Compiler<'a,'ctx>, float: IntValue<'ctx>)
{
    let func_name = "printf";
    let template_string: PointerValue<'ctx> = compiler.builder.build_global_string_ptr("%d", "glob_float_print").unwrap().as_pointer_value();
    let func = compiler.module.get_function(func_name).unwrap();
    let args = &[BasicMetadataValueEnum::from(template_string), BasicMetadataValueEnum::from(float)];
    let res = compiler.builder.build_call(func,args,func_name).unwrap();

}

mod tests
{

}

