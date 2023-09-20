use std::collections::HashMap;


    pub fn get_token_list(compilable_file: &str) -> Vec<Token>
    {
        let mut char_iter = compilable_file.chars();
        let mut token_iter = TokenIterator::new(char_iter);

        let result = token_iter.collect();
        println!("{:?}",result);
        result
    }

        ///Thus function returns the next token from the token iterator.
    pub fn get_next_token(compilable_file: String) -> Option<Token>
    {
        let mut char_iter = compilable_file.chars();
        let mut token_iter = TokenIterator::new(char_iter);
        token_iter.next()
    }
    
    //pub fn get_token_iterator(compilable_file: String) -> token_iterator<'_>
    //{
    //    let mut char_iter = compilable_file.chars();
    //    token_iterator::new(char_iter)
    //}
    pub struct TokenManager<'a>
    {
        pub current_token: Option<Token>,
        token_iter: TokenIterator<'a>
    }

    impl<'a> TokenManager<'a>
    {
        pub fn new(token_string: &str) -> TokenManager
        {
            let chars_over = TokenIterator::new(token_string.chars());
            
            let mut result = TokenManager { current_token: None, token_iter: chars_over };

            result.next_token();

            result
        }

        ///Thus function returns the next token from the token iterator.
        pub fn next_token(&mut self) -> &Option<Token>
        {
            self.current_token = self.token_iter.next();

            &self.current_token
        }
    }

    struct TokenIterator<'a> {
        char_iter: std::str::Chars<'a>,
        next_char: Option<char>,

    }
    impl<'a> TokenIterator<'a> {
        fn new(char_iter: std::str::Chars<'_>) -> TokenIterator
        {
            TokenIterator { 
                char_iter,
                next_char: Some(' '),//this is a space character. Don't touch.
            }
        }
        fn get_next_char(&mut self) -> Option<char>
        {
            self.next_char = self.char_iter.next();
            self.next_char
        }
        fn is_character_special(ch: char) -> bool
        {
            let special_chars = vec!['(',')','+','-','*',',','=',';'];
            
            if special_chars.contains(&ch)
            {
                true
            }
            else
            {
                false
            }
        }
    }
    impl Iterator for TokenIterator<'_> {
        type Item = Token;

        fn next(&mut self) -> Option<Self::Item>
        {
            if let None = self.next_char
            {
                return None;
            }
            dbg!("Running next!");
            let mut current_word_buffer = String::new();
            while let Some(current_character) = self.next_char
            {
                if current_word_buffer.is_empty() && current_character.is_whitespace()
                {

                    self.get_next_char();
                    println!("skipping over WHITESPACE {}",current_character);
                    continue;
                }
                let is_special = TokenIterator::is_character_special(current_character);
                
                

                //we have skipped over all the whitespace and are now building are buffer.
                if !current_character.is_whitespace() && !is_special
                {
                    current_word_buffer.push(current_character);
                }
                else if is_special && current_word_buffer.len() == 0
                {
                    current_word_buffer.push(current_character);
                    self.get_next_char();
                    break;
                }
                else if is_special
                {
                    break; //we dont need to get next char because itll be handled next iteration
                }
                else
                {
                    self.get_next_char();
                    break;
                }
                
                if current_character == ':'
                {
                    self.get_next_char();
                    return Some(Token::LABEL(current_word_buffer));
                }

                self.get_next_char();
                dbg!(self.next_char);
            }

        if let Ok(number) = current_word_buffer.parse()
            {
                return Some(Token::NumVal(number));
            }

    return Some(match current_word_buffer.to_uppercase().as_str()
            {
                "PROCEDURE" => Token::PROCEDURE,
                "PROC" => Token::PROCEDURE,
                ";" => Token::SEMICOLON,
                "," => Token::COMMA,
                "*" => Token::MULTIPLY,
                "(" => Token::OPEN_PAREN,
                ")" => Token::CLOSED_PAREN,
                "," => Token::COMMA,
                "/" => Token::DIVIDE,
                "+" => Token::PLUS,
                "-" => Token::MINUS,
                "DO" => Token::DO,
                "=" => Token::EQ,
                "PUT" => Token::PUT,
                "DATA" => Token::DATA,
                "END" => Token::END,
                "WHILE" => Token::WHILE,
                "SKIP" => Token::SKIP,
                "OPTIONS" => Token::OPTIONS,
                _ => Token::Identifier(current_word_buffer)
            });
        //}
            None
        }
        //{
        //    if let Some(cha) = self.semicolon_next
        //    {
        //        self.semicolon_next = None;
        //        return Some(
        //            match cha {
        //               ')' => Token::CLOSED_PAREN,
        //               '(' => Token::OPEN_PAREN,
        //               ',' => Token::COMMA,
        //               _ => Token::SEMICOLON,
        //            }
        //            );
        //    }
        //    let mut current_word_buffer = String::new();
        //    let mut string_mode = false; //if we found a open tick signifying string mode
        //    //we are looping until we find the first nonwhitespace.
        //    while let Some(next_char) = self.char_iter.next()
        //    {
        //        self.next_char = next_char;
        //        if self.next_char == '('
        //        {
        //            return Some(Token::OPEN_PAREN);
        //        }
        //        else if self.next_char == ')'
        //        {
        //            return Some(Token::CLOSED_PAREN);
        //        }
        //        else if self.next_char == ','
        //        {
        //            return Some(Token::COMMA);
        //        }
        //        else if self.next_char == '/'
        //        {
        //            let next_next_char = self.char_iter.next()?;
        //            if next_next_char == '*' //we are in a comment block
        //            {
        //                let mut found_second_star = false;
        //                while let Some(next_char) = self.char_iter.next()
        //                {
        //                    if next_char == '*'
        //                    {
        //                        found_second_star = true;
        //                    }
        //                    else if next_char == '/' && found_second_star
        //                    {
        //                        break;
        //                    }
        //                }
        //            }
        //            else if next_next_char.is_whitespace() //necessary cuz we eat the next next
        //                                                   //token
        //            {
        //                return Some(Token::DIVIDE);
        //            }
        //        }
        //        else if !self.next_char.is_whitespace()
        //        {
        //            if self.next_char == '\''
        //            {
        //                string_mode = true;
        //            }
        //            current_word_buffer.push(self.next_char);
        //            break;
        //        }
        //    }
        //    
        //    //now we are looping through characters until we find
        //    //a special ending character like a semicolon, colon,
        //    //or another whitespace and handle accordingly.
        //    while let Some(next_char) = self.char_iter.next()
        //    {
        //        self.next_char = next_char;
        //        if self.next_char.is_whitespace() && !string_mode
        //        {
        //            break;
        //        }
        //        else if string_mode && self.next_char == '\''
        //        {
        //            return Some(Token::STRING(String::from(&current_word_buffer[1..])));
        //        }
        //        else if self.next_char == ';'
        //        { //the current word ends in a semicolon
        //          //we break now, not loading the semicolon to the
        //          //string, and set a boolean so we can return a 
        //          //semicolon token the very next iteration.
        //            self.semicolon_next = Some(';');
        //            break;
        //        }
        //        else if self.next_char == ')'
        //        {
        //            self.semicolon_next = Some(')');
        //            break;
        //        }
        //        else if self.next_char == '('
        //        {
        //            self.semicolon_next = Some('(');
        //            break;
        //        }
        //        else if self.next_char == ','
        //        {
        //            self.semicolon_next = Some(',');
        //            break;
        //        }
        //        else if self.next_char == ':'
        //        {
        //            //current token ends in a semicolon, we can return
        //            //the current token early knowing its a label definition.
        //            return Some(Token::LABEL(current_word_buffer));
        //        }
        //        current_word_buffer.push(self.next_char);
        //    }
        //    println!("The word is: {}", current_word_buffer);
        //    

        //    //the token has been parsed, now we return
        //    //the proper token type.
        //    if current_word_buffer == ""
        //    { //no valid chars were added, must be EOF.
        //        return None;
        //    }
        //    if let Ok(number) = current_word_buffer.parse()
        //    {
        //        return Some(Token::NumVal(number));
        //    }
        //    Some(match current_word_buffer.to_uppercase().as_str()
        //    {
        //        "PROCEDURE" => Token::PROCEDURE,
        //        "PROC" => Token::PROCEDURE,
        //        ";" => Token::SEMICOLON,
        //        "," => Token::COMMA,
        //        "*" => Token::MULTIPLY,
        //        "/" => Token::DIVIDE,
        //        "+" => Token::PLUS,
        //        "-" => Token::MINUS,
        //        "DO" => Token::DO,
        //        "=" => Token::EQ,
        //        "PUT" => Token::PUT,
        //        "DATA" => Token::DATA,
        //        "END" => Token::END,
        //        "WHILE" => Token::WHILE,
        //        "SKIP" => Token::SKIP,
        //        "OPTIONS" => Token::OPTIONS,
        //        _ => Token::Identifier(current_word_buffer)
        //    })
        //}


        }
    

    #[derive(Debug, PartialEq,Clone)]
    pub enum Token{
        EOF,
        OPEN_PAREN,
        CLOSED_PAREN,
        PROCEDURE, // the procedure or proc token
        DO,
        PLUS,
        MINUS,
        MULTIPLY,
        DIVIDE,
        WHILE,
        END,
        PUT,
        LABEL(String),
        SKIP,
        STRING(String),
        EQ,
        SEMICOLON,
        COMMA,
        DATA,
        GET,
        OPTIONS,
        LIST,
        NumVal(i32), // integer
        Identifier(String), //an identifier / variable name

    }




    mod tests{
        use std::fs;

        use super::*;
        use super::Token::*;
        
        #[test]
        fn basic_parse()
        {
            let input = "FLAG = 0;";

            let output: Vec<Token> = vec![Token::Identifier("FLAG".to_string()), Token::EQ, Token::NumVal(0),Token::SEMICOLON];

        assert_eq!(output,get_token_list(input));            

        }
        #[test]
        fn special_char_good()
        {
            assert_eq!(TokenIterator::is_character_special('(') , true);
        }
        #[test]
        fn special_char_bad()
        {
            assert_eq!(TokenIterator::is_character_special(' ') , false);
        }
        #[test]
        fn hello_world_parse()
        {
            let input = fs::read_to_string("./hello_world.pli").unwrap();
            let output: Vec<Token> = vec![LABEL(String::from("HELLO")), PROCEDURE,
             OPTIONS, OPEN_PAREN, Identifier(String::from("MAIN")),CLOSED_PAREN,
            SEMICOLON,Identifier(String::from("FLAG")),EQ,NumVal(0),SEMICOLON,
            LABEL(String::from("LOOP")), DO, WHILE, OPEN_PAREN, Identifier(String::from("FLAG")), EQ, NumVal(0),
             CLOSED_PAREN, SEMICOLON, PUT, SKIP, DATA, OPEN_PAREN, STRING(String::from("HELLO WORLD!")),
              CLOSED_PAREN, SEMICOLON, END, Identifier(String::from("LOOP")), SEMICOLON, END, 
              Identifier(String::from("HELLO")), SEMICOLON
            ];

            assert_eq!(output, get_token_list(&input));
        }
        #[test]
        fn hello_world_with_commentparse()
        {
            let input = format!("{}{}","/*This is a comment!*/",fs::read_to_string("./hello_world.pli").unwrap());
            let output: Vec<Token> = vec![LABEL(String::from("HELLO")), PROCEDURE,
             OPTIONS, OPEN_PAREN, Identifier(String::from("MAIN")),CLOSED_PAREN,
            SEMICOLON,Identifier(String::from("FLAG")),EQ,NumVal(0),SEMICOLON,
            LABEL(String::from("LOOP")), DO, WHILE, OPEN_PAREN, Identifier(String::from("FLAG")), EQ, NumVal(0),
             CLOSED_PAREN, SEMICOLON, PUT, SKIP, DATA, OPEN_PAREN, STRING(String::from("HELLO WORLD!")),
              CLOSED_PAREN, SEMICOLON, END, Identifier(String::from("LOOP")), SEMICOLON, END, 
              Identifier(String::from("HELLO")), SEMICOLON
            ];

            assert_eq!(output, get_token_list(&input));
        }

        #[test]
        fn eq_with_statement()
        {
            let input = "/*A program to assign a variable*/A = 4;";
            let output = vec![Identifier(String::from("A")),EQ,NumVal(4),SEMICOLON];

            assert_eq!(get_token_list(input),output);
        }

        #[test]
        fn test_lexing_function_call()
        {
let input = "MIN(2,3);";
            let output = vec![Identifier(String::from("MIN")),OPEN_PAREN,NumVal(2),COMMA,NumVal(3),CLOSED_PAREN,SEMICOLON];

            assert_eq!(get_token_list(input),output);

        }

        #[test]
        fn test_binary_operator_lexing()
        {let input = "+ / / + + - *";
            let output = vec![PLUS, DIVIDE, DIVIDE, PLUS, PLUS, MINUS, MULTIPLY];

            assert_eq!(get_token_list(input),output);
        }
        #[test]
        fn test_touching_binary_lexing()
        {let input = "2+2";
            let output = vec![NumVal(2), PLUS, NumVal(2)];

            assert_eq!(get_token_list(input),output);
        }
    }




