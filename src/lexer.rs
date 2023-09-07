
    pub fn get_token_list(compilable_file: &str) -> Vec<Token>
    {
        let mut char_iter = compilable_file.chars();
        let mut token_iter = token_iterator::new(char_iter);

        let result = token_iter.collect();
        println!("{:?}",result);
        result
    }

    pub fn get_next_token(compilable_file: String) -> Option<Token>
    {
        let mut char_iter = compilable_file.chars();
        let mut token_iter = token_iterator::new(char_iter);
        token_iter.next()
    }
    
    //pub fn get_token_iterator(compilable_file: String) -> token_iterator<'_>
    //{
    //    let mut char_iter = compilable_file.chars();
    //    token_iterator::new(char_iter)
    //}
    struct token_iterator<'a> {
        char_iter: std::str::Chars<'a>,
        next_char: char,
        semicolon_next: Option<char>,

    }
    impl<'a> token_iterator<'a> {
        fn new(char_iter: std::str::Chars<'_>) -> token_iterator
        {
            token_iterator { 
                char_iter,
                next_char: ' ',//this is a space character. Don't touch.
                semicolon_next: None //stores whether a semicolon was loaded and should
                                      //immediately be returned.
            }
        }
    }
    impl Iterator for token_iterator<'_> {
        type Item = Token;

        fn next(&mut self) -> Option<Self::Item>
        {
            if let Some(cha) = self.semicolon_next
            {
                self.semicolon_next = None;
                return Some(
                    match cha {
                       ')' => Token::CLOSED_PAREN,
                       '(' => Token::OPEN_PAREN,
                       _ => Token::SEMICOLON,
                    }
                    );
            }
            let mut current_word_buffer = String::new();
            let mut string_mode = false; //if we found a open tick signifying string mode
            //we are looping until we find the first nonwhitespace.
            while let Some(next_char) = self.char_iter.next()
            {
                self.next_char = next_char;
                if self.next_char == '('
                {
                    return Some(Token::OPEN_PAREN);
                }
                else if self.next_char == ')'
                {
                    return Some(Token::CLOSED_PAREN);
                }
                else if self.next_char == '/'
                {
                    let next_next_char = self.char_iter.next()?;
                    if next_next_char == '*' //we are in a comment block
                    {
                        let mut found_second_star = false;
                        while let Some(next_char) = self.char_iter.next()
                        {
                            if next_char == '*'
                            {
                                found_second_star = true;
                            }
                            else if next_char == '/' && found_second_star
                            {
                                break;
                            }
                        }
                    }
                }
                else if !self.next_char.is_whitespace()
                {
                    if self.next_char == '\''
                    {
                        string_mode = true;
                    }
                    current_word_buffer.push(self.next_char);
                    break;
                }
            }
            
            //now we are looping through characters until we find
            //a special ending character like a semicolon, colon,
            //or another whitespace and handle accordingly.
            while let Some(next_char) = self.char_iter.next()
            {
                self.next_char = next_char;
                if self.next_char.is_whitespace() && !string_mode
                {
                    break;
                }
                else if string_mode && self.next_char == '\''
                {
                    return Some(Token::STRING(String::from(&current_word_buffer[1..])));
                }
                else if self.next_char == ';'
                { //the current word ends in a semicolon
                  //we break now, not loading the semicolon to the
                  //string, and set a boolean so we can return a 
                  //semicolon token the very next iteration.
                    self.semicolon_next = Some(';');
                    break;
                }
                else if self.next_char == ')'
                {
                    self.semicolon_next = Some(')');
                    break;
                }
                else if self.next_char == '('
                {
                    self.semicolon_next = Some('(');
                    break;
                }
                else if self.next_char == ':'
                {
                    //current token ends in a semicolon, we can return
                    //the current token early knowing its a label definition.
                    return Some(Token::LABEL(current_word_buffer));
                }
                current_word_buffer.push(self.next_char);
            }
            println!("The word is: {}", current_word_buffer);
            

            //the token has been parsed, now we return
            //the proper token type.
            if current_word_buffer == ""
            { //no valid chars were added, must be EOF.
                return None;
            }
            if let Ok(number) = current_word_buffer.parse()
            {
                return Some(Token::NumVal(number));
            }
            Some(match current_word_buffer.as_str() 
            {
                "PROCEDURE" => Token::PROCEDURE,
                "PROC" => Token::PROCEDURE,
                ";" => Token::SEMICOLON,
                "DO" => Token::DO,
                "=" => Token::EQ,
                "PUT" => Token::PUT,
                "DATA" => Token::DATA,
                "END" => Token::END,
                "WHILE" => Token::WHILE,
                "SKIP" => Token::SKIP,
                "OPTIONS" => Token::OPTIONS,
                _ => Token::Identifier(current_word_buffer)
            })
        }


        }
    

    #[derive(Debug, PartialEq)]
    pub enum Token{
        EOF,
        OPEN_PAREN,
        CLOSED_PAREN,
        PROCEDURE, // the procedure or proc token
        DO,
        WHILE,
        END,
        PUT,
        LABEL(String),
        SKIP,
        STRING(String),
        EQ,
        SEMICOLON,
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
    }




