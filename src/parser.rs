use crate::lexer::{self, Token};

#[derive(Debug)]
pub enum Expr
{
    Binary {
        operator: lexer::Token,
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
pub struct Prototype {

        pub fn_name: String,
        pub args: Vec<String> // the names of the arguments - used inside of the function itself.
}

pub struct Function{
    pub proto: Prototype,
    pub body: Expr
}



pub fn parse_numeric<'a>(token_manager: &'a mut lexer::TokenManager) -> Expr
{
    if let Some(Token::NumVal(value)) = token_manager.current_token
    {
        token_manager.next_token();//loads the next token into the token manager.
        return Expr::NumVal { value  };
    }
    else {
        panic!("Failed to parse numeric!");
    }
}

//parses identifiers like variable names but also function calls
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

///the current token is a '(' / Token::OPEN_PAREN
pub fn parse_parenthesis_expression(token_manager: &mut lexer::TokenManager) -> Expr {
   token_manager.next_token();
   let result: Expr = parse_expression(token_manager);
    
   if token_manager.current_token != Some(Token::CLOSED_PAREN)
   {
       panic!("Missing closed parenthesis on parenthesis expression!");
   }

   return result;
}

pub fn parse_expression<'a>(token_manager: &'a mut lexer::TokenManager) -> Expr {
    let left_handed_side = parse_primary_expression(token_manager);
    return match token_manager.current_token {
        Some(Token::PLUS) | Some(Token::MINUS) | Some(Token::DIVIDE) | Some(Token::MULTIPLY) => {
             let token_precedence = get_binary_operator_precedence(token_manager.current_token.as_ref().unwrap());
           // let right_handed_side = parse_right_side_of_binary_expression(token_manager, token_precedence);

           // return Expr::Binary 
           // { 
           //     operator: Token::PLUS,
           //     left: Box::new(left_handed_side), 
           //     right: Box::new(right_handed_side) 
           // }
           build_recursive_binary_tree(token_manager, left_handed_side, token_precedence )
        }
        _ =>  left_handed_side
    }
}

pub fn build_recursive_binary_tree(token_manager: &mut lexer::TokenManager, LHS: Expr, precendence: i32) -> Expr {
    //LHS has to be a binary node.
    let operator_token: Token = token_manager.current_token.as_ref().unwrap().clone(); 
    token_manager.next_token();
    //if the current precedence is GREATER than the lhs precendence,
    if let Expr::Binary { operator, left, right } = LHS {
        if precendence > get_binary_operator_precedence(&operator)
        {                        // meaning we have to make the RHS side of the LHS the LHS of our
                                 // new RHS
            let new_rhs = parse_expression(token_manager);

            //take 2 + 3 * 5
            //normally it will look like a tree of
            //           /   3
            //    bin - >  - +
            //           \   2
            //
            //    but we will make it look like
            //
            //           /   (3 * 5)
            //    bin - >  - +
            //           \   2
            let inner_binary = Expr::Binary { operator: operator_token, left: right, right: Box::new(new_rhs) };
            return Expr::Binary { operator, left, right: Box::new(inner_binary) };
        }
        else
        {
            let rhs_expression = parse_expression(token_manager);
            Expr::Binary { operator: operator_token, left: Box::new(Expr::Binary { operator, left, right }), right: Box::new(rhs_expression)}
        }
    }
    else
    {
        let right = parse_expression(token_manager);
        return Expr::Binary { operator: operator_token, left: Box::new(LHS), right: Box::new(right) };
    }
}
pub fn parse_primary_expression(token_manager: &mut lexer::TokenManager) -> Expr
{
    match token_manager.current_token.as_ref().unwrap() {
    Token::OPEN_PAREN => parse_parenthesis_expression(token_manager),
    Token::Identifier(_) => parse_identifier(token_manager),
    Token::NumVal(_) => parse_numeric(token_manager),
    other => panic!("Can't parse another token type!")
    }
}

pub fn get_binary_operator_precedence(token: &lexer::Token) -> i32
{
   match token {
    Token::PLUS => 20,
    Token::MINUS => 20,
    Token::MULTIPLY => 40,
    Token::DIVIDE => 40,
    _ => -1
   } 
}

//The token is currently PROCEDURE
//CALC: PROCEDURE(A,B,C); // we are just parsing this part.
//      RETURN(A+B+C);
//      END;
pub fn parse_function_prototype(token_manager: &mut lexer::TokenManager, label_name: String) -> Prototype
{
         token_manager.next_token();
         println!("Begging to parse function proto!");
         //token should now be open paren
         if Some(Token::OPEN_PAREN) != token_manager.current_token
         {
             panic!("Was expecting an open parenthesis!");
         }
            token_manager.next_token();//go inside the parenthesis
            let mut expecting_comma = false;
            let mut args_list: Vec<String> = vec![];
           loop {
                println!("Looping!");
                if let Some(Token::CLOSED_PAREN) = token_manager.current_token
                {
                    println!("found a closed parenthesis!");
                    token_manager.next_token();// eat the closed parenthesis token, ready for next use
                    //token_manager.next_token();// eat the semicolon after the closed paren 
                    parse_semicolon(token_manager);
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

                    let arg_name: String;
                    if let Expr::Variable { name } = parsed_arg
                    {
                        arg_name = name.clone();
                    }
                    else
                    {
                        panic!("Expected variable in function prototype, found _!");
                    }

                    args_list.push(arg_name);

                    expecting_comma = true;
                    println!("turned expecting comma on!");
                }
                else 
                {
                    panic!("{:?}",token_manager.current_token);
                }

                
           }

    Prototype { fn_name: label_name, args: args_list }
}

pub fn parse_function(token_manager: &mut lexer::TokenManager) -> Function
{
    let proto = parse_function_prototype(token_manager, String::from("TESTFUNC")); 
    let exp = parse_expression(token_manager);

    parse_semicolon(token_manager);//eat the trailing semicolon
                                   //
    if token_manager.current_token != Some(Token::END)
    {
        panic!("{} is missing an END tag!", proto.fn_name);
    }

    token_manager.next_token();
   Function { proto, body: exp } 
}

///parses the beginning of a PL/1 Program.
///They look like this:
/// ANY_LABEL_HERE : PROCDURE OPTIONS (MAIN);
pub fn parse_opening(token_manager: &mut lexer::TokenManager){
   if let Some(Token::LABEL(_)) = token_manager.current_token
   {
       token_manager.next_token();
   }
   else
   {
       panic!("Program not beginning with a label!");
   }
    if let Some(Token::PROCEDURE) = token_manager.current_token
   {
       token_manager.next_token();
   }
   else
   {
       panic!("Program missing main proc");
   }
     if let Some(Token::OPTIONS) = token_manager.current_token
   {
       token_manager.next_token();
   }
   else
   {
       panic!("Program missing OPTIONS attribute on main procedure!");
   }
      if let Some(Token::OPEN_PAREN) = token_manager.current_token
   {
       token_manager.next_token();
   }
   else
   {
       panic!("Program missing OPEN PAREN on main procedure!");
   }
   if let Some(Token::Identifier(ref var)) = token_manager.current_token
   {
       if var == "MAIN"
       {
           token_manager.next_token();
       }
       else
       {
           panic!("Option in main procedure is not MAIN");
       }
   }
   else
   {
       panic!("Program missing MAIN OPTION on main procedure!");
   }
    if let Some(Token::CLOSED_PAREN) = token_manager.current_token
   {
       token_manager.next_token();
   }
   else
   {
       panic!("Program missing CLOSED PAREN on main procedure!");
   }
   parse_semicolon(token_manager);
}

pub fn parse_semicolon(token_manager: &mut lexer::TokenManager)
{
    if let Some(Token::SEMICOLON) = token_manager.current_token
    {
        token_manager.next_token();
    }
    else
    {
        panic!("Expected semicolon!");
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
           operator: Token::PLUS,
           left: Box::new(LHS),
           right: Box::new(RHS),
       };

       let LHSVar = Expr::Variable { name: String::from("x") };
       
       let RHSVar = Expr::Variable { name: String::from("y") };
        
       let test = Expr::Binary {
           operator: Token::PLUS,
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

        let mut tok_man = TokenManager::new("4");

        let result: Expr = parse_numeric(&mut tok_man);
        
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
            assert_eq!(Token::SEMICOLON, tok_man.current_token.unwrap());
        }
        else
        {
            panic!("Was not a call Expr");
        }
    }

    #[test]
    fn test_parse_parenthesis_expression() 
    {
        let mut tok_man = TokenManager::new("(25665)");

        let result: Expr = parse_parenthesis_expression(&mut tok_man);

        if let Expr::NumVal{value} =  result
        {
            assert_eq!(25665,value);
        }
        else
        {
            panic!("NOT A NUMVAL!");
        }
    }

    #[test]
    #[should_panic]
    fn test_parse_paren_bad_syntax()
    {
        let mut tok_man = TokenManager::new("(2 min(2,3))");
        
        let result: Expr = parse_parenthesis_expression(&mut tok_man);
    }

    #[test]
    fn test_parse_primaries()
    {
        let mut tok_man = TokenManager::new("2; MIN(9,254); FLAG; (4);");

        
            let result = parse_expression(&mut tok_man);
            tok_man.next_token();
            if let Expr::NumVal { value } = result
            {
                assert_eq!(value,2);
            }
            else { panic!("Not a numval 2!"); }

            let result = parse_expression(&mut tok_man);
            tok_man.next_token();
            if let Expr::Call { fn_name, args } = result
            {
                assert_eq!("MIN", fn_name);
            }
            else { panic!("Not a MIN func!"); }

            let result = parse_expression(&mut tok_man);
            tok_man.next_token();
            
            if let Expr::Variable {name} = result
            {
                assert_eq!("FLAG", name);
            }
            else { panic!("Not a variable named FLAG!"); }

            let result = parse_expression(&mut tok_man);
            tok_man.next_token();

            if let Expr::NumVal { value } = result
            {
                assert_eq!(4, value);
            }
            else { panic!("Not a numval of value 4!"); }

    }

    #[test]
    fn test_binaries()
    {
        let mut token_manager = TokenManager::new("2 + 2");
        let result = parse_expression(&mut token_manager);

        if let Expr::Binary { operator, left, right } = result
        {
            assert_eq!(Token::PLUS, operator);

            let left_expr: Expr = *left;

            if let Expr::NumVal { value } = left_expr{
                assert_eq!(value, 2);
            }
            else
            {
                panic!("not numval");
            }

            let right_expr: Expr = *right;
            if let Expr::NumVal { value } = right_expr{
                assert_eq!(value, 2);
            }
            else
            {
                panic!("not numval");
            }
        }
        else
        {
            panic!("Expression was not a binary, was a {:?}", result);
        }

        //2. nested binaries
        //
        let mut token_manager = TokenManager::new("2 + 3 * 5");
        let result = parse_expression(&mut token_manager);

        if let Expr::Binary { operator, left, right } = result
        { // this is the 2 in 2 + 3 * 5
            assert_eq!(Token::PLUS, operator);

            let left_expr: Expr = *left;

            if let Expr::NumVal { value } = left_expr{
                assert_eq!(value, 2);
            }
            else
            {
                panic!("not numval");
            }

            let right_expr: Expr = *right; // this is the 3 * 5 in 2 + 3 * 5
            if let Expr::Binary { operator, left, right } = right_expr{
                
                if let Expr::NumVal { value } = *left{
                    assert_eq!(3, value);
                }   
                else { panic!("not a numval!")}
                
                if let Token::MULTIPLY  = operator{
                }   
                else { panic!("not a multiply!")}
                
                if let Expr::NumVal { value } = *right{
                    assert_eq!(5, value);
                }   
                else { panic!("not a numval!")}
            }
            else
            {
                panic!("not numval");
            }
        }
        else
        {
            panic!("Expression was not a binary, was a {:?}", result);
        }
    }

    #[test]
    fn test_parsing_prototype()
    {
        let mut token_manager = TokenManager::new("PROCEDURE(A,B,C);");
        let my_var: Prototype = parse_function_prototype(&mut token_manager,String::from("CALC"));

        assert_eq!(String::from("CALC"), my_var.fn_name);
        assert_eq!(my_var.args.len(), 3);
        
        let test_results = vec!["A","B","C"];
        let mut index = 0;
        for (siz,arg) in my_var.args.iter().enumerate()
        {
            assert_eq!(*arg,String::from(test_results[index]));
            index += 1;
            
        }


    }
    #[test]
    #[should_panic(expected = "open paren")]
    fn test_parsing_prototype_panic()
    {
        let mut token_manager = TokenManager::new("PROCEDURE A,B,C);");
        let my_var: Prototype = parse_function_prototype(&mut token_manager,String::from("CALC"));

        assert_eq!(String::from("CALC"), my_var.fn_name);
        assert_eq!(my_var.args.len(), 3);
        
        let test_results = vec!["A","B","C"];
        let mut index = 0;
        for (siz,arg) in my_var.args.iter().enumerate()
        {
            assert_eq!(*arg,String::from(test_results[index]));
            index += 1;
            
        }

    }
    #[test]
    fn test_parsing_prototype_noargs()
    {
        let mut token_manager = TokenManager::new("PROCEDURE();");
        let my_var: Prototype = parse_function_prototype(&mut token_manager,String::from("CALC"));

        assert_eq!(String::from("CALC"), my_var.fn_name);
        assert_eq!(my_var.args.len(), 0);
    }
    #[test]
    fn test_parsing_function()
    {
        let mut token_manager = TokenManager::new("PROCEDURE (A,B,C); A + B + C; END;");

        let my_function = parse_function(&mut token_manager);



    }
}

