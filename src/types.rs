use inkwell::{types::StructType, context::Context};

use crate::codegen::codegen::Compiler;

use self::fixed_decimal::Test;

/// Holds all type data
mod fixed_decimal;
//DCL (A,B,C,D,E) FIXED(3);


// a FIXED DECIMAL is a i16
// a FLOAT DECIMAL is a floating point i16 number
// FIXED DECIMAL (3) or (3,0) means a number with 3 digits before period i.e 100
// FIXED DECIMAL (3.1) means a number with 3 digits before period, one after i.e 100.1


//FIXED BINARY -> use LLVM's APInt data type: arbitrary precision integers
//FIXED DECIMAL -> use LLVM's APInt data type: arbitrary precision integers
//BINARY FLOAT -> use double like we are currently using
//DECIMAL FLOAT -> use double like we are currently using
pub enum BaseAttributes {
    DECIMAL, //if you specify only decimal, then float is assumed too
    FLOAT,
    FIXED, //if you speecify only fixed, then decimal is assumed too
}


#[derive(Clone,Debug)]
pub enum Type
{
    FixedDecimal,
    Float,
}


#[derive(Clone,Debug)]
pub enum FixedRadix
{
    Decimal,
    Binary,
}




#[derive(Debug,Clone)]
pub struct TypeModule<'ctx>
{
    fixed_type: StructType<'ctx>,
}



impl<'ctx> TypeModule<'ctx>
{
    pub fn new(ctx: &'ctx Context) -> Self
    {
        TypeModule { fixed_type: fixed_decimal::get_fixed_type(ctx) }
    }
}







impl<'a,'ctx> Compiler<'a,'ctx>
{
    fn load_types(&'a self)
    {
    }
}
























#[derive(Clone,Debug)]
pub struct Fixed
{
    radix: FixedRadix,
    digits_before_decimal: Vec<u8>,
    digits_after_decimal: Vec<u8>,
    is_negative: bool,
}

impl Fixed
{

}

/// the 0th index in the array is the one closest to the decimal
impl Default for Fixed
{
    fn default() -> Self
    {
        Fixed {
            
            radix: FixedRadix::Decimal,
            digits_before_decimal: vec![],
            digits_after_decimal: vec![],
            is_negative: false,

        }
    }
}

impl From<i64> for Fixed
{
    fn from(value: i64) -> Self
    {
        let lol = Test::Yes;
        let mut value = value; 
        let mut before_decimal: Vec<u8> = vec![];
        let mut is_neg = false;

        if value < 0
        {
            is_neg = true;
            value *= -1;
        }

        loop {
            let current_digit: u8 = (value % 10) as u8;
            before_decimal.push(current_digit);
            
            value = value / 10;

            if value == 0
            {
                break;
            }


        }
       
        let mut return_val = Fixed::default(); 
        return_val.digits_before_decimal = before_decimal;
        return_val.is_negative = is_neg;
        return_val
    }
}


impl Into<i64> for Fixed
{
    fn into(self) -> i64 {
        match self.radix
        {
            FixedRadix::Decimal => {
                
                let mut result: i64 = 0;

                for (i, digit) in self.digits_before_decimal.into_iter().enumerate()
                {
                   result += digit as i64 * (i64::pow(10,i as u32));
                }

                if self.is_negative
                {
                    result *= -1;
                }

                result 
                
            },
            other =>
            {
                todo!("Cannot handle into for anything but decimal!");
            }
        }
    }
}


mod tests {
    use crate::types::fixed_decimal::Test;

    use super::Fixed;


    #[test]
    fn testy()
    {
        let initial_val: i64 = 426;
        let fixed_val = Fixed::from(initial_val);
        
        let converted_val: i64 = fixed_val.into();

        
        assert_eq!(initial_val,converted_val);

    }

    #[test]
    fn testy_neg()
    {
        let initial_val = -602;
        let fixed_val = Fixed::from(initial_val);
        
        let converted_val: i64 = fixed_val.into();

        
        assert_eq!(initial_val,converted_val);
        let lol = Test::Yes;
    }
}