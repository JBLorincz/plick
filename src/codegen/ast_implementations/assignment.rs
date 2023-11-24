use std::error::Error;

use inkwell::values::{BasicValue, BasicValueEnum, AnyValue, PointerValue};
use crate::{codegen::{codegen::{CodeGenable, Compiler}, named_value_store::NamedValueStore, named_value::NamedValue, utils}, ast::{self, Expr}, types::{character::{CharValue, generate_character_code_for_size}, Type}};

impl<'a, 'ctx> CodeGenable<'a,'ctx> for ast::Assignment
{
    unsafe fn codegen(self, compiler: &'a crate::codegen::codegen::Compiler<'a, 'ctx>)
                -> Box<dyn inkwell::values::AnyValue<'ctx> + 'ctx> {
            self.codegen_with_error_info(compiler).expect("Error generating ASSIGNMENT")
    }
}

impl<'a, 'ctx> ast::Assignment
{
    #[allow(warnings)]
  unsafe fn codegen_with_error_info(
            &self,
            compiler: &'a crate::codegen::codegen::Compiler<'a, 'ctx>
        ) -> Result<Box<dyn AnyValue<'ctx> + 'ctx>,Box<dyn Error>> {

            let variable_in_map = compiler.named_values.try_get(&self.var_name);

            log::debug!("generating assignment code for variable {:#?}",variable_in_map);

            let var_ptr: PointerValue<'ctx>; //= _named_value.pointer;
                                             
            let _type = match variable_in_map.clone()
            {
                Some(val) => val._type,
                None => self.value.get_type(compiler)
            }; 
                                             
            let current_func = utils::get_current_function(compiler);

            var_ptr = match variable_in_map.clone()
            {
                Some(val) => val.pointer,
                None => 
                {
                    let ptr = compiler.create_entry_block_alloca(&self.var_name, &current_func, &_type);
                    let named_value = NamedValue { name: self.var_name.clone(), _type, pointer: ptr };
                    compiler.named_values.insert(named_value);
                    ptr
                }
            };


              let value_to_store: Box<dyn AnyValue> = 
                  expr_assignment_gen::codegen_expr_assignment(self.value.clone(), &_type, compiler);

                    let initial_value: BasicValueEnum<'ctx> =
                        compiler.convert_anyvalue_to_basicvalue(value_to_store);

                    let _store_result = compiler
                        .builder
                        .build_store(var_ptr, initial_value).unwrap();

                    return Ok(Box::new(initial_value));
        }
}

mod expr_assignment_gen
{
    use inkwell::values::AnyValue;

    use crate::{ast::Expr, types::{Type, character::{CharValue, generate_character_code_for_size}}, codegen::{named_value::NamedValue, codegen::{Compiler, CodeGenable}}};

    pub unsafe fn codegen_expr_assignment<'a, 'ctx>(value: Expr, _type: &Type, compiler: &Compiler<'a,'ctx>)
        -> Box<dyn AnyValue<'ctx> + 'ctx>
    {
                    let value_to_store: Box<dyn AnyValue>;

                    value_to_store = match value.clone()
                    {
                        Expr::Char { value } => codegen_char_assignment(&value, *_type, compiler),
                        _other => codegen_default_assignment(value, compiler),
                    };

                    value_to_store
    }


    unsafe fn codegen_char_assignment<'a, 'ctx>(value: &str, _type: Type, compiler: &Compiler<'a,'ctx>)
        -> Box<dyn AnyValue<'ctx> + 'ctx>
    {                   
                        let mut siz = 0;
                        if let Type::Char(size) = _type
                        {
                            siz = size;
                        }
                        else
                        {
                            log::error!("Trying to generate a char but the type of the variable was {:#?} instead!", _type);
                        }
                        let char_val: CharValue<'ctx> = generate_character_code_for_size(compiler.context, &value, siz);
                        Box::new(char_val.value.as_any_value_enum())
    }


    unsafe fn codegen_default_assignment<'a, 'ctx>(value: Expr, compiler: &Compiler<'a,'ctx>)
        -> Box<dyn AnyValue<'ctx> + 'ctx>
    {
        value.clone().codegen(compiler)
    }
}
