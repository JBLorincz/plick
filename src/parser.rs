use crate::lexer::{self, Token};


pub enum Expr<'a>
{
    Binary {
        operator: char,
        left: Box<Expr<'a>>,
        right: Box<Expr<'a>>
    },

    Call {
        fn_name: String,
        args: Vec<Expr<'a>>
    },
    NumVal {
        value: i32
    },
    Conditional {
        
    },
    Variable {
        name: &'a str
    }

        
}

//Represents a function prototype
struct Prototype<'a> {

        fn_name: &'a str,
        args: Vec<&'a str> // the names of the arguments - used inside of the function itself.
}

struct Function<'a>{
    proto: Prototype<'a>,
    body: Expr<'a>
}



pub fn parse_numeric(numeric_token: &lexer::Token) -> Expr
{
    if let Token::NumVal(value) = numeric_token
    {
        Expr::NumVal { value: *value }
    }
    else {
        panic!("Failed to parse numeric!");
    }
}



mod tests {

    use super::*;

    #[test]
    fn construct_binary(){
        let LHS = Expr::NumVal { value: 4 };
        let RHS = Expr::NumVal { value: 6 };

       let test = Expr::Binary {
           operator: '+',
           left: Box::new(LHS),
           right: Box::new(RHS),
       };

       let LHSVar = Expr::Variable { name: "x" };
       
       let RHSVar = Expr::Variable { name: "y" };
        
       let test = Expr::Binary {
           operator: '+',
           left: Box::new(LHSVar),
           right: Box::new(RHSVar),
       };   

       let LHSVar = Expr::Variable { name: "x" };
       if let Expr::Variable { name } = LHSVar
       {
            assert_eq!(name, "x");
       }
       else
       {
           panic!("panicking here!");
       }

    }


    #[test]
    fn test_parsing_numeric()
    {
        let my_token = lexer::Token::NumVal(4);
        let result: Expr = parse_numeric(&my_token);
        
       if let Expr::NumVal{value: val} = result
       {
            assert_eq!(4,val);
       }
       else {
           panic!("Result of parse numeric was not a numeric expression!");
       }
    }

}
