use std::error::Error;

use inkwell::values::{IntValue, BasicMetadataValueEnum};

use crate::codegen::codegen::{CodeGenable, Compiler};
use crate::ast::{self, Expr};
use crate::codegen::named_value_store::NamedValueStore;


impl<'a, 'ctx> CodeGenable<'a,'ctx> for ast::Get
{
    unsafe fn codegen(self, compiler: &'a crate::codegen::codegen::Compiler<'a, 'ctx>)
                -> Box<dyn inkwell::values::AnyValue<'ctx> + 'ctx> {
                    
                    let _res = compiler.generate_get_code(self.list_to_get).unwrap();
                    Box::new(compiler.generate_float_code(-999.0))
        
    }
}


impl<'a,'ctx> Compiler<'a,'ctx>
{
pub unsafe fn generate_get_code(
            &self,
            list: ast::IOList,
        ) -> Result<(), Box<dyn Error>> {
            log::trace!("Calling generate get code!");

            let mut result: IntValue<'ctx>;
            for i in list.items.iter()
            {
                log::debug!("{:#?}",i);
                if let Expr::Variable { _type, name }  = i
                {
                    log::debug!("Running get loop for variable {}",name);
                    let does_var_exist: bool = self
                        .named_values.try_get(name).map(|v| true).unwrap_or(false);

                    log::trace!("Does value exist? {}",does_var_exist);
                    let real_type = self
                        .named_values
                        .try_get(name)
                        .map(|value| value._type)
                        .unwrap_or(_type.clone());

                    log::trace!("getting variable {} of type {}",name,real_type);

                    let format_string = &Self::get_format_string_for_type(&real_type);
                    let format_string_ptr = self.builder.build_global_string_ptr(format_string, "format_string")?.as_pointer_value();
                    
                    let scanf_func = self.get_function("scanf")?;

                    let variable_ptr = self.create_or_load_variable(name, &real_type);


                    let mut args :Vec<BasicMetadataValueEnum> = vec![];
                    args.push(format_string_ptr.into());
                    args.push(variable_ptr.into());
                    
                    let scanf_return_value  = self.builder.build_call(scanf_func, &args[..], "scanf")?;
                    result = scanf_return_value.try_as_basic_value().left().unwrap().into_int_value();
                }
                else
                {
                    panic!("Expected a variable in the GET LIST, recieved a {:#?}",i);
                }
            }
            Ok(())
        }
}
