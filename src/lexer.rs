
    pub fn hello_world(compilable_file: String) -> Vec<Token>
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

    struct token_iterator<'a> {
        char_iter: std::str::Chars<'a>,

    }
    impl<'a> token_iterator<'a> {
        fn new(char_iter: std::str::Chars<'_>) -> token_iterator
        {
            token_iterator { char_iter }
        }
    }
    impl Iterator for token_iterator<'_> {
        type Item = Token;

        fn next(&mut self) -> Option<Self::Item>
        {
            let mut next_char_result = self.char_iter.next();
            if let None = next_char_result
            {
                return None;
            }
            //from here on we can 
            let mut current_word_buffer = String::new();

            let mut next_char = next_char_result.unwrap();
            
            //skip all the beginning white space.
           while next_char.is_whitespace() 
           {
               next_char_result = self.char_iter.next();
               next_char = next_char_result?;
           }
            while !next_char.is_whitespace()
            {
                current_word_buffer.push(next_char);
                
                next_char_result = self.char_iter.next();
                if let None = next_char_result
                {
                    return None;
                }
                if next_char == ':'
                {
                    return Some(Token::LABEL(current_word_buffer));
                }
                next_char = next_char_result.unwrap();
            }

            println!("The word was: '{}'",current_word_buffer);
            
            Some(match current_word_buffer.as_str() 
            {
                "PROCEDURE" => Token::PROCEDURE,
                "PROC" => Token::PROCEDURE,
                "DO" => Token::DO,
                "=" => Token::EQ,
                "PUT" => Token::PUT,
                "END" => Token::END,
                "WHILE" => Token::WHILE,
                "SKIP" => Token::SKIP,
                "OPTIONS" => Token::OPTIONS,
                _ => Token::Identifier(current_word_buffer)
            })
            
            


        }
    }

    #[derive(Debug)]
    pub enum Token{
        EOF,
        PROCEDURE, // the procedure or proc token
        DO,
        WHILE,
        END,
        PUT,
        LABEL(String),
        SKIP,
        EQ,
        DATA,
        GET,
        OPTIONS,
        LIST,
        NumVal(i32), // integer
        Identifier(String), //an identifier / variable name

    }
