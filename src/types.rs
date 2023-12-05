use std::{
    fmt::Display,
    mem::{discriminant, Discriminant},
};

use inkwell::{
    context::Context,
    types::{AnyType, AnyTypeEnum, ArrayType, BasicType, BasicTypeEnum, StructType},
    values::{FloatValue, IntValue, PointerValue, StructValue},
};
use log::debug;

use crate::{codegen::codegen::Compiler, error::get_error};

use self::fixed_decimal::{generate_fixed_decimal_code, FixedValue};

pub mod character;
/// Holds all type data
pub mod fixed_decimal;
pub mod traits;
const SIZE_OF_STRINGS: u32 = 255;

//DCL (A,B,C,D,E) FIXED(3);

// a FIXED DECIMAL is a i16
// a FLOAT DECIMAL is a floating point i16 number
// FIXED DECIMAL (3) or (3,0) means a number with 3 digits before period i.e 100
// FIXED DECIMAL (3.1) means a number with 3 digits before period, one after i.e 100.1

//FIXED BINARY -> use LLVM's APInt data type: arbitrary precision integers
//FIXED DECIMAL -> use LLVM's APInt data type: arbitrary precision integers
//BINARY FLOAT -> use double like we are currently using
//DECIMAL FLOAT -> use double like we are currently using
#[derive(Debug, Clone)]
pub enum BaseAttributes {
    DECIMAL, //if you specify only decimal, then float is assumed too
    FLOAT,
    FIXED, //if you speecify only fixed, then decimal is assumed too
}

#[derive(Clone, Debug, Copy, PartialEq, PartialOrd)]
pub enum Type {
    ///Our custom FixedValue struct
    FixedDecimal,
    ///Just a Inkwell FloatValue
    Float,
    /// Not a type: just represents something
    /// whose type has to be determined later.
    TBD,
    ///The return type of some functions
    Void,
    ///The string type
    Char(u32),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Into<String> for Type {
    fn into(self) -> String {
        format!("{}", self)
    }
}

#[derive(Clone, Debug)]
pub enum FixedRadix {
    Decimal,
    Binary,
}

#[derive(Debug, Clone)]
pub struct TypeModule<'ctx> {
    pub fixed_type: StructType<'ctx>,
}

///Takes two input types, and determines what the output type should be.
///Useful for binary expressions.
pub fn resolve_types(type_one: &Type, type_two: &Type) -> Result<Type, String> {
    if *type_one == Type::TBD
        || *type_two == Type::TBD
        || *type_one == Type::Void
        || *type_two == Type::Void
    {
        return Err(get_error(&[
            "5",
            &type_one.to_string(),
            &type_two.to_string(),
        ]));
    }

    if *type_one == *type_two {
        return Ok(type_one.clone());
    }

    //by this point we have one fixed decimal and one float.
    Ok(Type::FixedDecimal)
}

pub fn do_types_match(_type1: &Type, _type2: &Type) -> bool {
    let lhs: Discriminant<Type> = discriminant(_type1);
    let rhs: Discriminant<Type> = discriminant(_type2);

    lhs == rhs
}

impl<'ctx> TypeModule<'ctx> {
    pub fn new(ctx: &'ctx Context) -> Self {
        TypeModule {
            fixed_type: fixed_decimal::get_fixed_type(ctx),
        }
    }
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    fn load_types(&'a self) {}

    pub fn get_character_type(&self, size: u32) -> ArrayType<'ctx> {
        character::get_character_type(self.context, size)
    }

    pub fn gen_const_fixed_decimal(&self, value: f64) -> StructValue<'ctx> {
        let struc: StructValue<'ctx> =
            generate_fixed_decimal_code(self.context, self.type_module.fixed_type, value).into();

        struc
    }

    pub fn convert_plick_type_to_llvm_basic_type(&'a self, _type: Type) -> BasicTypeEnum<'ctx> {
        let result = match _type {
            Type::FixedDecimal => self.type_module.fixed_type.as_basic_type_enum(),
            Type::Char(size) => self.get_character_type(size).as_basic_type_enum(),
            Type::Float => todo!("implement float type"),
            Type::Void => panic!("Can't convert void type to basic type enum!"),
            Type::TBD => panic!("Can't convert TBD type to basic type enum!"),
        };

        debug!("Type {} turned into {}", _type, result);

        result
    }
    pub fn convert_plick_type_to_llvm_any_type(&'a self, _type: Type) -> AnyTypeEnum<'ctx> {
        match _type {
            Type::FixedDecimal => self.type_module.fixed_type.as_any_type_enum(),
            Type::Char(size) => self.get_character_type(size).as_any_type_enum(),
            Type::Float => todo!("implement float type"),
            Type::Void => self.context.void_type().as_any_type_enum(),
            Type::TBD => panic!("Can't convert TBD type to any type enum!"),
        }
    }
}

///The attributes of returned values may be delcared in two ways:
///1. They may be declared by default according to the first letter of the function name. For
///   example, if the function begings with the letters A through H or O through Z, then the result
///   will be DECIMAL FLOAT(6), because that is the default attribute of identifiers beginning with
///   those letters. Function names beginning with the letters I through N return a result with the
///   attributes FIXED BINARY(15).
///
///   2. Because the default attributes for function names do not allow us to return a result that
///      is FIXED DECIMAL or FLOAT DECIMAL(16), for example, we have another method of specifying
///      the attributes of a returned value. This is accomplished through the RETURNS keyword.
pub fn infer_pli_type_via_name(name_of_pli_object: &str) -> Type {
    let first_letter_of_func = name_of_pli_object
        .chars()
        .next()
        .unwrap()
        .to_ascii_lowercase();

    if 105 < first_letter_of_func as u32 {
        return Type::FixedDecimal;
    } else {
        return Type::FixedDecimal;
    }

    todo!("Make it so all functions don't return just a fixed decimal by default!")
}
