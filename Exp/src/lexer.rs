use crate::token::Token;

pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            if token != Token::Whitespace {
                tokens.push(token);
            }
        }
        tokens
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        if self.position >= self.input.len() {
            return None;
        }

        let current_char = self.input[self.position];

        if current_char.is_alphabetic() {
            return Some(self.read_identifier_or_keyword());
        }

        if current_char.is_numeric() {
            return Some(self.read_number());
        }

        if current_char == '"' {
            return Some(self.read_string_literal());
        }

        if current_char == '\'' {
            return Some(self.read_char_literal());
        }

        if current_char == '/' {
            if self.position + 1 < self.input.len() {
                match self.input[self.position + 1] {
                    '/' => {
                        self.position += 2;
                        self.skip_single_line_comment();
                        return self.next_token();
                    }
                    '*' => {
                        self.position += 2;
                        self.skip_multi_line_comment();
                        return self.next_token();
                    }
                    _ => {}
                }
            }
        }

        if "+-*/=<>!&|".contains(current_char) {
            return Some(self.read_operator());
        }

        self.position += 1;
        Some(Token::Symbol(current_char))
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.input[self.position].is_whitespace() {
            self.position += 1;
        }
    }

    fn skip_single_line_comment(&mut self) {
        while self.position < self.input.len() && self.input[self.position] != '\n' {
            self.position += 1;
        }
    }

    fn skip_multi_line_comment(&mut self) {
        while self.position + 1 < self.input.len() {
            if self.input[self.position] == '*' && self.input[self.position + 1] == '/' {
                self.position += 2;
                break;
            }
            self.position += 1;
        }
    }

    fn read_identifier_or_keyword(&mut self) -> Token {
        let start = self.position;
        while self.position < self.input.len() && self.input[self.position].is_alphanumeric() {
            self.position += 1;
        }
        let identifier: String = self.input[start..self.position].iter().collect();
        match identifier.as_str() {
            "int" | "char" | "float" | "double" | "void" => Token::DataType(identifier),
            "if" | "else" | "while" | "for" | "break" | "continue" => Token::ControlStructure(identifier),
            _ => Token::Identifier(identifier),
        }
    }

    fn read_number(&mut self) -> Token {
        let start = self.position;
        while self.position < self.input.len() && self.input[self.position].is_numeric() {
            self.position += 1;
        }
        Token::Number(self.input[start..self.position].iter().collect())
    }

    fn read_string_literal(&mut self) -> Token {
        self.position += 1; // Skip the opening quote
        let start = self.position;
        while self.position < self.input.len() && self.input[self.position] != '"' {
            self.position += 1;
        }
        let string_literal: String = self.input[start..self.position].iter().collect();
        self.position += 1; // Skip the closing quote
        Token::StringLiteral(string_literal)
    }

    fn read_char_literal(&mut self) -> Token {
        self.position += 1; // Skip the opening quote
        if self.position < self.input.len() - 1 && self.input[self.position + 1] == '\'' {
            let char_literal = self.input[self.position];
            self.position += 2; // Skip the char and the closing quote
            Token::CharLiteral(char_literal)
        } else {
            Token::Unknown('\'')
        }
    }

    fn read_operator(&mut self) -> Token {
        let start = self.position;
        let current_char = self.input[start];
        self.position += 1;

        if self.position < self.input.len() {
            let next_char = self.input[self.position];
            match (current_char, next_char) {
                ('=', '=') => {
                    self.position += 1;
                    return Token::Equals;
                }
                ('!', '=') => {
                    self.position += 1;
                    return Token::NotEquals;
                }
                ('&', '&') => {
                    self.position += 1;
                    return Token::And;
                }
                ('|', '|') => {
                    self.position += 1;
                    return Token::Or;
                }
                _ => {}
            }
        }

        match current_char {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Multiply,
            '/' => Token::Divide,
            '=' => Token::Assign,
            '<' => Token::LessThan,
            '>' => Token::GreaterThan,
            _ => Token::Unknown(current_char),
        }
    }
}