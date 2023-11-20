use std::error::Error;

use crate::{codegen::codegen::CodeGenable, ast, types::infer_pli_type_via_name};



impl<'a,'ctx> CodeGenable<'a,'ctx> for ast::Declare
{
    unsafe fn codegen(self, compiler: &'a crate::codegen::codegen::Compiler<'a, 'ctx>)
                -> Box<dyn inkwell::values::AnyValue<'ctx> + 'ctx> {

                    self.codegen_with_error_info(compiler).expect("Error generating DECLARE statement")
        
    }
}

impl<'a, 'ctx> ast::Declare
{
    unsafe fn codegen_with_error_info(self, compiler: &'a crate::codegen::codegen::Compiler<'a, 'ctx>)
                -> Result<Box<dyn inkwell::values::AnyValue<'ctx> + 'ctx>, Box<dyn Error>> {

            log::info!("Generating declare code!");
            let name = self.var_name.clone(); 
            log::info!("Name: {}",name);
            let _type = self.attribute.unwrap_or(infer_pli_type_via_name(&name));

            log::info!("Type: {}",_type);
            //let current_function = get_current_function(self);
            //self.create_entry_block_alloca(&name, &current_function, &_type)
            Ok(Box::new(compiler.create_or_load_variable(&name, &_type)))

    }
}
