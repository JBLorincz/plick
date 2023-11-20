use std::error::Error;

use inkwell::values::{BasicValue, BasicValueEnum, AnyValue};

use crate::{codegen::{codegen::{CodeGenable, Compiler}, named_value_store::NamedValueStore, named_value::NamedValue}, ast};

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

            match variable_in_map {

                Some(_pointer_value) => {
                    let value_to_store = self.value.clone().codegen(compiler);

                    let initial_value: BasicValueEnum<'ctx> =
                        compiler.convert_anyvalue_to_basicvalue(value_to_store);

                    let _store_result = compiler
                        .builder
                        .build_store(_pointer_value.pointer, initial_value);

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
            let _type = assignment.value.get_type();
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
