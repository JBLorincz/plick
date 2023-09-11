use crate::lexer::{self, Token};

#[derive(Debug)]
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
        println!("The identifier string is: {}",identifier_string);
   }
   else {
       panic!("failed to parse identifier!");
   }
        let mut args_list: Vec<Expr> = vec![];
       token_manager.next_token();// prime next token
       if let Some(Token::OPEN_PAREN) = token_manager.current_token
       {
           println!("Found an open parenthesis first!");
           //function call here.
           //now we loop through each expression in the arguments
           let mut expecting_comma: bool = false;// expecting comma does not affect breaking
            println!("Turning expecting comma off!"); 
            token_manager.next_token();
           loop {
                println!("Looping!");
                if let Some(Token::CLOSED_PAREN) = token_manager.current_token
                {
                    println!("found a closed parenthesis!");
                    token_manager.next_token();// eat the next token, ready for next use
                    break;
                }
                else if let Some(Token::COMMA) = token_manager.current_token 
                {
                    if expecting_comma
                    {
                        println!("Found comma at right place, continuing!");
                        expecting_comma = false;
                        token_manager.next_token();// eat the token

                    }
                    else 
                    {
                        panic!("Expected an expression, found a comma!!");     
                    }
                }
                else if let Some(ref Token) = token_manager.current_token
                {
                    //parse as expression
                    println!("Found a token called {:#?}", *Token);
                    let parsed_arg: Expr = parse_expression(token_manager);

                    args_list.push(parsed_arg);

                    expecting_comma = true;
                    println!("turned expecting comma on!");
                }
                else 
                {
                    panic!("{:?}",token_manager.current_token);
                }

                
           }
            return Expr::Call { fn_name: identifier_string, args: args_list };
       }
       else 
       {
            return Expr::Variable { name: identifier_string}; 
       }
   
   
}

pub fn parse_expression<'a>(token_manager: &'a mut lexer::TokenManager) -> Expr {
    if let Some(Token::NumVal(value)) = token_manager.current_token
    {
        token_manager.next_token();
        return Expr::NumVal { value };
    }
   Expr::Variable { name: String::from("test") } 
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

    #[test]
    fn test_parsing_identifier()
    {
        let mut tok_man = TokenManager::new("MIN(2,3);");
        let result = parse_identifier(&mut tok_man);
        if let Expr::Call{fn_name, args} = result
        {
            assert_eq!(fn_name,"MIN");
            assert_eq!(args.len(),2);

            if let Expr::NumVal { value } = args[0]
            {
                assert_eq!(value,2);
            }
            else
            {
                panic!("args[0] was not type numval");
            }
        }
        else
        {
            panic!("Was not a call Expr");
        }
    }
}

