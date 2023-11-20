use inkwell::{AddressSpace, types::{BasicMetadataTypeEnum, PointerType, FunctionType, FloatType}, module};

use super::codegen::Compiler;





pub fn add_extern_functions<'a, 'ctx>(compiler: &mut Compiler<'a, 'ctx>)
{
    let printf_arg_type: PointerType<'ctx> =
        compiler.context.i8_type().ptr_type(AddressSpace::default());

    let printf_type: FunctionType<'ctx> = compiler
        .context
        .i32_type()
        .fn_type(&[BasicMetadataTypeEnum::from(printf_arg_type)], true);

    let _printf_func =
        compiler
            .module
            .add_function("printf", printf_type, Some(module::Linkage::DLLImport));

    let double_type: FloatType<'ctx> =
        compiler.context.f64_type();

    let pow_type: FunctionType<'ctx> = compiler
        .context
        .f64_type()
        .fn_type(&[BasicMetadataTypeEnum::from(double_type),BasicMetadataTypeEnum::from(double_type)], false);

    let _pow_func =
        compiler
            .module
            .add_function("pow", pow_type, Some(module::Linkage::DLLImport));



        let scanf_arg_type: PointerType<'ctx> =
        compiler.context.i8_type().ptr_type(AddressSpace::default());

        let scanf_type: FunctionType<'ctx> = compiler
        .context
        .i32_type()
        .fn_type(&[BasicMetadataTypeEnum::from(scanf_arg_type)], true);

    let _scanf_func =
        compiler
            .module
            .add_function("scanf", scanf_type, Some(module::Linkage::DLLImport));
 
}
