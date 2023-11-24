use std::error::Error;

use inkwell::values::{BasicValue, BasicValueEnum, AnyValue};

use crate::{codegen::{codegen::{CodeGenable, Compiler}, named_value_store::NamedValueStore, named_value::NamedValue}, ast::{self, Expr}, types::{character::{CharValue, generate_character_code_for_size}, Type}};

impl<'a, 'ctx> CodeGenable<'a,'ctx> for ast::Assignment
{
    unsafe fn codegen(self, compiler: &'a crate::codegen::codegen::Compiler<'a, 'ctx>)
                -> Box<dyn inkwell::values::AnyValue<'ctx> + 'ctx> {
            self.codegen_with_error_info(compiler).expect("Error generating ASSIGNMENT")
    }
}

impl<'a, 'ctx> ast::Assignment
{
  unsafe fn codegen_with_error_info(
            &self,
            compiler: &'a crate::codegen::codegen::Compiler<'a, 'ctx>
        ) -> Result<Box<dyn AnyValue<'ctx> + 'ctx>,Box<dyn Error>> {

            let variable_in_map = compiler.named_values.try_get(&self.var_name);
            log::debug!("Assigning variable {:#?}",variable_in_map);
            if &self.var_name == "MYLOL"
            {
                //panic!("WHY?");
            }
            match variable_in_map {

                Some(_named_value) => {
                    //Problem with chars that aren't the right size?

                    let value_to_store: Box<dyn AnyValue>;
                    //TODO CLEAN ALL THIS UP USE A TRAIT FOR GENNING VALUES MAYBE?
                    if let Expr::Char { value } = self.value.clone()
                    {
                        let mut siz = 0;
                        if let Type::Char(size) = _named_value._type
                        {
                            siz = size;
                        }
                        let char_val: CharValue = generate_character_code_for_size(compiler.context, &value, siz);
                        value_to_store = Box::new(char_val.value.as_any_value_enum());
                        
                    }
                    else
                    {
                        value_to_store = self.value.clone().codegen(compiler);
                    }
                    

                    let initial_value: BasicValueEnum<'ctx> =
                        compiler.convert_anyvalue_to_basicvalue(value_to_store);

                    let _store_result = compiler
                        .builder
                        .build_store(_named_value.pointer, initial_value).unwrap();

                    return Ok(Box::new(initial_value));
                }

                None => {
                    let new_variable = compiler.create_variable_from_assignment(self.clone());
                    Ok(Box::new(new_variable.as_any_value_enum()))
                }
            }
        }
}


impl<'a,'ctx> Compiler<'a,'ctx>
{
        unsafe fn create_variable_from_assignment(
            &self,
            assignment: ast::Assignment,
        ) -> Box<dyn BasicValue<'ctx> + 'ctx> {
            let _type = assignment.value.get_type(self);
            dbg!(&_type);
            let name = assignment.var_name.clone();

            dbg!(&assignment);

            let variable_ptr = self.allocate_variable(&assignment);

            dbg!(&variable_ptr);
            let value_of_variable = self.assign_variable(assignment, variable_ptr);
            self.named_values
                .insert(NamedValue::new(name, _type, variable_ptr));

            Box::new(value_of_variable)
        }
}
