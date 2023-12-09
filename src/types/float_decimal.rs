use super::traits::{Mathable, MathableFactory, Puttable};
use crate::codegen::codegen;
use crate::codegen::{codegen::Compiler, utils::print_float_value};
use inkwell::{
    types::{BasicType, BasicTypeEnum, StructType},
    values::{BasicValue, BasicValueEnum, FloatValue, IntValue, StructValue},
};

#[derive(Debug)]
pub struct PLIFloatDecimalValue<'ctx> {
    pub value: StructValue<'ctx>,
}

impl<'ctx> PLIFloatDecimalValue<'ctx> {
    pub fn new(value: StructValue<'ctx>) -> PLIFloatDecimalValue<'ctx> {
        PLIFloatDecimalValue { value }
    }
    pub fn get_llvm_basic_type<'a>(compiler: &Compiler<'a, 'ctx>) -> BasicTypeEnum<'ctx> {
        get_float_decimal_type(compiler).as_basic_type_enum()
    }

    pub fn set_precision<'a>(
        &self,
        _value: IntValue<'ctx>,
        _compiler: &Compiler<'a, 'ctx>,
    ) -> Self {
        todo!("Make this actually work and return a self value!");
        let malloc_ptr = _compiler
            .builder
            .build_malloc(self.value.get_type(), "malloc_to_get_float")
            .unwrap();

        _compiler
            .builder
            .build_store(malloc_ptr, self.value)
            .unwrap();

        let precision_ptr = _compiler
            .builder
            .build_struct_gep(malloc_ptr, 1, "gepping_to_precision")
            .unwrap();

        _compiler
            .builder
            .build_store(precision_ptr, _value)
            .unwrap();

        _compiler.builder.build_free(malloc_ptr).unwrap();
    }
}

impl<'a, 'ctx> Puttable<'a, 'ctx> for PLIFloatDecimalValue<'ctx> {
    unsafe fn print_object(&self, compiler: &'a Compiler<'a, 'ctx>) {
        let malloc_ptr = compiler
            .builder
            .build_malloc(self.value.get_type(), "malloc_to_get_float")
            .unwrap();

        compiler
            .builder
            .build_store(malloc_ptr, self.value)
            .unwrap();

        let precision_ptr = compiler
            .builder
            .build_struct_gep(malloc_ptr, 0, "gepping_to_float")
            .unwrap();

        let struct_float = compiler
            .builder
            .build_load(precision_ptr, "loading_float")
            .unwrap()
            .into_float_value();

        print_float_value(compiler, struct_float);

        compiler.builder.build_free(malloc_ptr).unwrap();
    }
    fn get_pointer_to_printable_string(
        &self,
        compiler: &'a Compiler<'a, 'ctx>,
    ) -> inkwell::values::PointerValue<'ctx> {
        todo!("Implement way to get pointer!");
    }
}

impl<'a, 'ctx> Mathable<'a, 'ctx> for PLIFloatDecimalValue<'ctx> {
    fn convert_to_float(&self, compiler: &'a Compiler<'a, 'ctx>) -> FloatValue<'ctx> {
        let struc: StructValue<'ctx> = self.value;

        let malloc_ptr = compiler
            .builder
            .build_malloc(struc.get_type(), "malloc_for_floatie")
            .unwrap();

        compiler
            .builder
            .build_store(malloc_ptr, self.value)
            .unwrap();

        let float_val: FloatValue<'ctx>;

        let float_ptr = compiler
            .builder
            .build_struct_gep(malloc_ptr, 0, "gepping_to_float")
            .unwrap();

        float_val = compiler
            .builder
            .build_load(float_ptr, "getf64_from_float_decimal")
            .unwrap()
            .into_float_value();

        compiler.builder.build_free(malloc_ptr).unwrap();
        float_val
    }
}

impl<'a, 'ctx> MathableFactory<'a, 'ctx, PLIFloatDecimalValue<'ctx>>
    for PLIFloatDecimalValue<'ctx>
{
    unsafe fn create_mathable(
        float: &FloatValue<'ctx>,
        compiler: &Compiler<'a, 'ctx>,
    ) -> Box<PLIFloatDecimalValue<'ctx>> {
        let mut values: Vec<BasicValueEnum> = vec![];
        values.push(
            compiler
                .context
                .f64_type()
                .const_zero()
                .as_basic_value_enum(),
        );
        values.push(
            compiler
                .context
                .i8_type()
                .const_zero()
                .as_basic_value_enum(),
        );

        let newstruc = get_float_decimal_type(compiler).const_named_struct(&values);
        let ptr = compiler
            .builder
            .build_alloca(newstruc.get_type(), "float_type")
            .unwrap();

        let float_ptr = compiler
            .builder
            .build_struct_gep(ptr, 0, "storing float in pli float value")
            .unwrap();
        compiler
            .builder
            .build_store(float_ptr, float.as_basic_value_enum())
            .unwrap();
        let newstruc = compiler
            .builder
            .build_load(ptr, "loading new float")
            .unwrap()
            .into_struct_value();

        Box::new(PLIFloatDecimalValue::new(newstruc))
    }
}

fn get_float_decimal_type<'a, 'ctx>(compiler: &Compiler<'a, 'ctx>) -> StructType<'ctx> {
    let mut field_types: Vec<BasicTypeEnum> = vec![];
    field_types.push(compiler.context.f64_type().as_basic_type_enum());
    field_types.push(compiler.context.i8_type().as_basic_type_enum());
    compiler.context.struct_type(&field_types, false)
}
