use std::error::Error;

use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::{BasicMetadataValueEnum, IntValue};
use inkwell::AddressSpace;

use crate::ast::{self, Expr};
use crate::codegen::codegen::{CodeGenable, Compiler};
use crate::codegen::named_value_store::NamedValueStore;
use crate::codegen::utils::print_float_value;
use crate::types::fixed_decimal::FixedValue;
use crate::types::traits::MathableFactory;
use crate::types::Type;

impl<'a, 'ctx> CodeGenable<'a, 'ctx> for ast::Get {
    unsafe fn codegen(
        self,
        compiler: &'a crate::codegen::codegen::Compiler<'a, 'ctx>,
    ) -> Box<dyn inkwell::values::AnyValue<'ctx> + 'ctx> {
        let _res = compiler.generate_get_code(self.list_to_get).unwrap();
        Box::new(compiler.generate_float_code(-999.0))
    }
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    pub unsafe fn generate_get_code(&self, list: ast::IOList) -> Result<(), Box<dyn Error>> {
        log::trace!("Calling generate get code!");

        let mut result: IntValue<'ctx>;
        for i in list.items.iter() {
            log::debug!("{:#?}", i);
            if let Expr::Variable { _type, name } = i {
                log::debug!("Running get loop for variable {}", name);
                let does_var_exist: bool = self
                    .named_values
                    .try_get(name)
                    .map(|v| true)
                    .unwrap_or(false);

                log::trace!("Does value exist? {}", does_var_exist);
                let real_type = self
                    .named_values
                    .try_get(name)
                    .map(|value| value._type)
                    .unwrap_or(_type.clone());

                log::trace!("getting variable {} of type {}", name, real_type);

                let format_string = &Self::get_format_string_for_type(&real_type);
                let format_string_ptr = self
                    .builder
                    .build_global_string_ptr(format_string, "format_string")?
                    .as_pointer_value();

                let scanf_func = self.get_function("scanf")?;

                let final_variable_ptr = self.create_or_load_variable(name, &real_type);

                let type_of_tmp_scan_var =
                    self.determine_scanf_type_from_plick_type(real_type.clone());
                let tmp_scan_tr = self
                    .builder
                    .build_malloc(type_of_tmp_scan_var, "what")
                    .unwrap();

                let mut args: Vec<BasicMetadataValueEnum> = vec![];
                args.push(format_string_ptr.into());
                args.push(tmp_scan_tr.into());

                let scanf_return_value = self.builder.build_call(scanf_func, &args[..], "scanf")?;

                //now load variable_ptr with tmp_scan_tr

                match &real_type {
                    Type::FixedDecimal => {
                        let scanned_float_value = self
                            .builder
                            .build_load(tmp_scan_tr, "load scanned")
                            .unwrap()
                            .into_float_value();
                        let x: Box<FixedValue<'ctx>> =
                            FixedValue::create_mathable(&scanned_float_value, self);
                        self.builder
                            .build_store(final_variable_ptr, x.value)
                            .unwrap();
                    }
                    Type::Char(_size) => {
                        let scanned_chars = self
                            .builder
                            .build_load(tmp_scan_tr, "load scanned")
                            .unwrap()
                            .into_array_value();
                        self.builder
                            .build_store(final_variable_ptr, scanned_chars)
                            .unwrap();
                    }
                    _ => {
                        panic!("Don't know how to get this type!");
                    }
                }

                //end

                //now we need
                result = scanf_return_value
                    .try_as_basic_value()
                    .left()
                    .unwrap()
                    .into_int_value();
            } else {
                panic!("Expected a variable in the GET LIST, recieved a {:#?}", i);
            }
        }
        Ok(())
    }

    fn determine_scanf_type_from_plick_type(&self, _type: Type) -> BasicTypeEnum<'ctx> {
        match _type {
            Type::FixedDecimal => self.context.f64_type().into(),
            Type::Char(_size) => self.get_character_type(_size).into(),
            _ => panic!("Don't know how to scan this type!"),
        }
    }
}
