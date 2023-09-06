
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



fn parse_numeric(numeric_token: Expr)
{

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

    }

}
