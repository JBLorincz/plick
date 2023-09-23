use std::{collections::HashMap, error::Error};


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
            let special_chars = vec!['/','(',')','\'','+','-','*',',','=',';','<','>'];

            
            if special_chars.contains(&ch)
            {
                true
            }
            else
            {
                false
            }
        }
            //next_char is '
            fn process_string(&mut self, mut current_word_buffer: String) -> String
            {
                self.get_next_char(); //skip over the first tick
                while let Some(char) = self.next_char
                {
                    match char 
                    {
                        '\'' => break,
                        ch => current_word_buffer.push(ch)
                    }
                    self.get_next_char();
                    
                }
                self.get_next_char();
                current_word_buffer
            }

            //current char is *
            fn process_comment(&mut self) -> Option<()>
            {
                let mut found_second_star = false;
                loop
                {
                    let current_char = self.get_next_char()?;
                    if current_char == '*'
                    {
                        found_second_star = true;
                    }
                    else if current_char == '/' && found_second_star
                    {
                        self.get_next_char();
                        break;
                    }
                }
                Some(())
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
            let mut current_word_buffer = String::new();
            while let Some(current_character) = self.next_char
            {
                if current_word_buffer.is_empty() && current_character.is_whitespace()
                {

                    self.get_next_char();
                    continue;
                }
                let is_special = TokenIterator::is_character_special(current_character);
                
                if is_special && current_character == '\''
                {
                    current_word_buffer = self.process_string(current_word_buffer);
                    return Some(Token::STRING(current_word_buffer));
                }
                else if is_special && current_character == '/'
                {
                    println!("THIS IS THE CHAR!");
                    let next_lext_char = self.get_next_char();
                    let ch = next_lext_char?;

                    if ch == '*'
                    {
                        self.process_comment()?;
                        continue;
                    }
                    else
                    {
                        self.get_next_char();
                        return Some(Token::DIVIDE);
                    }

                }

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
                else //if the current character is whitespace
                {
                    self.get_next_char();
                    break;
                }
                
                if current_character == ':'
                {
                    self.get_next_char();
                    current_word_buffer.pop();
                    return Some(Token::LABEL(current_word_buffer));
                }

                self.get_next_char();
            }

        if let Ok(number) = current_word_buffer.parse()
            {
                return Some(Token::NumVal(number));
            }
            if current_word_buffer.len() == 0
            {
                return None;
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
                "<" => Token::LESS_THAN,
                ">" => Token::GREATER_THAN,
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
        }

        
    }
    

    #[derive(Debug, PartialEq,Clone)]
    pub enum Token
    {
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
        LESS_THAN,
        GREATER_THAN,
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




