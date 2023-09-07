use crate::lexer::{self, Token};


pub enum Expr
{
    Binary {
        operator: char,
        left: Box<Expr>,
        right: Box<Expr>
    },

    Call {
        fn_name: String,
        args: Vec<Expr>
    },
    NumVal {
        value: i32
    },
    Conditional {
        
    },
    Variable {
        name: String 
    }

        
}

//Represents a function prototype
struct Prototype<'a> {

        fn_name: &'a str,
        args: Vec<&'a str> // the names of the arguments - used inside of the function itself.
}

struct Function<'a>{
    proto: Prototype<'a>,
    body: Expr
}



pub fn parse_numeric<'a>(numeric_token: &lexer::Token, token_manager: &'a mut lexer::TokenManager) -> Expr
{
    if let Token::NumVal(value) = numeric_token
    {
        token_manager.next_token();//loads the next token into the token manager.
        return Expr::NumVal { value: *value  };
    }
    else {
        panic!("Failed to parse numeric!");
    }
}

pub fn parse_identifier<'a>(token_manager: &'a mut lexer::TokenManager) -> Expr{
     let identifier_string: String;
   if let Some(Token::Identifier(ref val)) = token_manager.current_token 
   {
        identifier_string = val.clone();
   }
   else {
       panic!("failed to parse identifier!");
   }
        let args_list: Vec<Expr> = vec![];
       token_manager.next_token();// prime next token
       if let Some(Token::OPEN_PAREN) = token_manager.current_token
       {
           //function call here.
           //now we loop through each expression in the arguments
           let expecting_comma: bool = false;
           loop {
               token_manager.next_token();

                if let Some(Token::CLOSED_PAREN) = token_manager.current_token
                {
                    break;
                }
                
           }
            return Expr::Call { fn_name: identifier_string, args: args_list };
       }
       else 
       {
            return Expr::Variable { name: identifier_string}; 
       }
   
   
}

mod tests {

    use crate::lexer::TokenManager;

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

       let LHSVar = Expr::Variable { name: String::from("x") };
       
       let RHSVar = Expr::Variable { name: String::from("y") };
        
       let test = Expr::Binary {
           operator: '+',
           left: Box::new(LHSVar),
           right: Box::new(RHSVar),
       };   

       let LHSVar = Expr::Variable { name: String::from("x") };
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

        let mut tok_man = TokenManager::new("");

        let result: Expr = parse_numeric(&my_token,&mut tok_man);
        
       if let Expr::NumVal{value: val} = result
       {
            assert_eq!(4,val);
       }
       else {
           panic!("Result of parse numeric was not a numeric expression!");
       }
    }
}

