mod node_dict;
mod compiler_error;

use node_dict::TokenType;
use compiler_error::CompilerError;

use regex::Regex;
use std::{fs::File, io::{self, BufRead}, iter::Peekable, ops::AddAssign, path::Path, str::SplitWhitespace};

// Check if a token is a valid identifier
fn is_identifier(token: &str) -> bool {
    let identifier_re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    identifier_re.is_match(token)
}

// Check if a token is a number
fn is_number(token: &str) -> bool {
    let number_re = Regex::new(r"^\d+(\.\d+)?$").unwrap();
    number_re.is_match(token)
}

// Tokenize a single line of code
fn tokenize_line(line: &str, in_multiline_comment: &mut bool) -> Vec<(String, TokenType)> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut chars = line.chars().peekable();
    let mut in_string = false;
    let mut in_char = false;
    let mut escape = false;

    while let Some(&ch) = chars.peek() {
        if *in_multiline_comment {
            if ch == '*' {
                chars.next();
                if let Some(&'/') = chars.peek() {
                    *in_multiline_comment = false;
                }
            }
            chars.next();
            continue;
        }

        if in_string {
            // current_token.push(ch); //Uncomment in case of needin' the quotes in the token type
            if ch == '"' && !escape {
                in_string = false;
                tokens.push((current_token.clone(), TokenType::String));
                current_token.clear();
            } else {
                current_token.push(ch); // Remove this in case of needin' the quotes in the token type
            }
            escape = ch == '\\' && !escape;
            chars.next();
            continue;
        }

        if in_char {
            // current_token.push(ch); //Uncomment in case of needin' the quotes in the token type
            if ch == '\'' && !escape {
                in_char = false;
                tokens.push((current_token.clone(), TokenType::Char));
                current_token.clear();
            } else {
                current_token.push(ch); // Remove this in case of needin' the quotes in the token type
            }
            escape = ch == '\\' && !escape;
            chars.next();
            continue;
        }

        match ch {
            '/' => {
                chars.next();
                if let Some(&next_ch) = chars.peek() {
                    if next_ch == '*' {
                        *in_multiline_comment = true;
                        chars.next();
                        continue;
                    } else if next_ch == '/' {
                        break; // Ignore the rest of the line
                    } else {
                        current_token.push('/');
                        current_token.push(next_ch);
                        chars.next();
                    }
                }
            }
            '"' => {
                if !current_token.is_empty() {
                    if node_dict::RESERVED_WORDS.contains(&current_token.as_str()) {
                        //tokens.push((current_token.clone(), TokenType::ReservedWord)); //TODO: Function to differentiate between reserved words
                    } else if is_identifier(&current_token) {
                        tokens.push((current_token.clone(), TokenType::Identifier));
                    } else if is_number(&current_token) { 
                        //tokens.push((current_token.clone(), TokenType::Number)); //TODO: Function to differentiate between int, float and double
                    } else {
                        //tokens.push((current_token.clone(), TokenType::Other));
                    }
                    current_token.clear();
                }
                //current_token.push(ch); //Uncomment in case of needin' the quotes in the token type
                in_string = true;
                chars.next();
            }
            '\'' => {
                if !current_token.is_empty() {
                    if node_dict::RESERVED_WORDS.contains(&current_token.as_str()) {
                        //tokens.push((current_token.clone(), TokenType::ReservedWord)); //TODO: Function to differentiate between reserved words
                    } else if is_identifier(&current_token) {
                        tokens.push((current_token.clone(), TokenType::Identifier));
                    } else if is_number(&current_token) {
                        //tokens.push((current_token.clone(), TokenType::Number)); //TODO: Function to differentiate between int, float and double
                    } else {
                        //tokens.push((current_token.clone(), TokenType::Other));
                    }
                    current_token.clear();
                }
                //current_token.push(ch); //Uncomment in case of needin' the quotes in the token type
                in_char = true;
                chars.next();
            }
            _ if ch.is_whitespace() => {
                if !current_token.is_empty() {
                    if node_dict::RESERVED_WORDS.contains(&current_token.as_str()) {
                        //tokens.push((current_token.clone(), TokenType::ReservedWord)); // TODO Function to differentiate between reserved words
                    } else if is_identifier(&current_token) {
                        tokens.push((current_token.clone(), TokenType::Identifier));
                    } else if is_number(&current_token) {
                        //tokens.push((current_token.clone(), TokenType::Number));//todo: function to differentiate between int, float and double
                    } else {
                        //tokens.push((current_token.clone(), TokenType::Other));
                    }
                    current_token.clear();
                }
                //tokens.push((" ".to_owned(), TokenType::Space)); // Add space as a token
                chars.next();
                continue;
            }
            _ if ch.is_digit(10)
                || (ch == '.' && current_token.chars().all(|c| c.is_digit(10))) =>
            {
                current_token.push(ch);
                chars.next();
                continue;
            }
            _ if ch.is_alphanumeric() || ch == '_' => {
                if !current_token.is_empty()
                    && !current_token
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_')
                {
                    if is_number(&current_token) {
                        //tokens.push((current_token.clone(), TokenType::Number)); //todo: Function to differentiate between int, float and double
                    } else if is_identifier(&current_token) {
                        tokens.push((current_token.clone(), TokenType::Identifier));
                    } else {
                        //tokens.push((current_token.clone(), TokenType::Other));
                    }
                    current_token.clear();
                }
                current_token.push(ch);
                chars.next();
                continue;
            }
            _ => {
                if !current_token.is_empty() {
                    if is_number(&current_token) {
                        //tokens.push((current_token.clone(), TokenType::Number)); //TODO: Function to differentiate between int, float and double
                    } else if is_identifier(&current_token) {
                        tokens.push((current_token.clone(), TokenType::Identifier));
                    } else {
                        //tokens.push((current_token.clone(), TokenType::Other));
                    }
                    current_token.clear();
                }

                match ch {
                    '=' => tokens.push((ch.to_string(), TokenType::Assign)),
                    ';' => tokens.push((ch.to_string(), TokenType::Semicolon)),
                    ':' => tokens.push((ch.to_string(), TokenType::Colon)),
                    ',' => tokens.push((ch.to_string(), TokenType::Comma)),
                    '(' => tokens.push((ch.to_string(), TokenType::LParen)),
                    ')' => tokens.push((ch.to_string(), TokenType::RParen)),
                    '{' => tokens.push((ch.to_string(), TokenType::LBrack)),
                    '}' => tokens.push((ch.to_string(), TokenType::RBrack)),
                    '\\' => tokens.push(("\\".to_owned(), TokenType::Backslash)),
                    // TODO ADD OPERATORS ONE BY ONE
                    _ => {
                        let mut operator = String::new();
                        operator.push(ch);
                        chars.next();

                        while let Some(&next_ch) = chars.peek() {
                            if next_ch.is_alphanumeric() || next_ch == '_' || next_ch == ' ' {
                                break;
                            }


                            operator.push(next_ch);
                            if !node_dict::OPERATORS_AND_SYMBOLS.contains(&operator.as_str()) {
                                operator.pop();
                                break;
                            }
                            chars.next();
                        }

                        if node_dict::OPERATORS_AND_SYMBOLS.contains(&operator.as_str()) {
                            //tokens.push((operator.clone(), TokenType::Operator));
                        } else {
                            //tokens.push((operator.clone(), TokenType::Other));
                        }
                        continue;
                    }
                }
                chars.next();
            }
        }
    }

    if !current_token.is_empty() {
        if node_dict::RESERVED_WORDS.contains(&current_token.as_str()) {
            //tokens.push((current_token.clone(), TokenType::ReservedWord)); // TODO Function to differentiate between reserved words
        } else if is_identifier(&current_token) {
            tokens.push((current_token.clone(), TokenType::Identifier));
        } else if is_number(&current_token) {
            //tokens.push((current_token.clone(), TokenType::Number)); // TODO Function to differentiate between int, float and double
        } /*else {
            tokens.push((current_token.clone(), TokenType::Other)); // TODO Error handling
        }*/
    }

    tokens
}

// Read the file and return lines
fn read_file(file_path: &str) -> io::Result<Vec<String>> {
    let path = Path::new(file_path);
    let file = File::open(&path)?;
    let lines = io::BufReader::new(file).lines();
    lines.collect()
}

// AST Node definitions
#[derive(Debug)]
enum ASTNode {
    VariableDeclaration {
        var_type: String,
        var_name: Vec<String>,
        var_value: Vec<Option<String>>,
    },
    VariableAssignment {
        var_name: String,
        value: String,
    },
    FunctionDefinition {
        return_type: String,
        func_name: String,
        parameters: Vec<(String, String)>,
        body: Vec<ASTNode>,
    },
    WhileLoop {
        condition: String,
        body: Vec<ASTNode>,
    },
    IfStatement {
        condition: String,
        body: Vec<ASTNode>,
        else_body: Option<Vec<ASTNode>>,
    },
    Block(Vec<ASTNode>),
    Expression(String),
}

// Parser definition
struct Parser {
    tokens: Peekable<std::vec::IntoIter<(String, TokenType)>>,
}

impl Parser {
    fn new(tokens: Vec<(String, TokenType)>) -> Self {
        Parser {
            tokens: tokens.into_iter().peekable(),
        }
    }

    fn parse(&mut self) -> Vec<ASTNode> {
        let mut ast = Vec::new();
        while let Some(node) = self.parse_statement() {
            ast.push(node);
        }
        ast
    }

    fn parse_statement(&mut self) -> Option<ASTNode> {
        if let Some((token, token_type)) = self.tokens.peek() {
            match token_type {
                TokenType::Integer | TokenType::Float | TokenType::Double | TokenType::Char => self.parse_declaration(),
                //TokenType::Void => self.parse_void_declaration(),
                TokenType::While => self.parse_while_loop(),
                TokenType::If => self.parse_if_statement(),
                TokenType::Identifier => self.parse_variable_assignment(),
                TokenType::LBrace => self.parse_block(),
                TokenType::RBrace => {
                    self.tokens.next();
                    None
                },
                _ => {
                    self.tokens.next();
                    None
                }
            }
        } else {
            None
        }
    }

    fn parse_declaration(&mut self) -> Option<ASTNode> {
        let var_type = self.tokens.next()?.0;
        let mut var_name = Vec::new();
        let mut var_value = Vec::new();

        //int a=8, b, c=9+5+10-12;

        // Expect a semicolon
        loop {
            let x = self.tokens.next();
            match x {
                Some ((token, TokenType::Semicolon)) => {
                    break;
                },
                Some ((token, TokenType::Identifier)) => {
                    var_name.push(token);
                    let (auxTok, auxTokType) = self.tokens.peek().unwrap();
                    if let TokenType::Comma = auxTokType {
                        var_value.push(None);
                        continue;
                    } else if let TokenType::Assign = auxTokType {
                        self.tokens.next();
                        let mut value = String::new();
                        while let Some((token, token_type)) = self.tokens.peek() {
                            if let TokenType::Comma = token_type {
                                break;
                            } else {
                                value.push_str(token);
                                self.tokens.next();
                            }
                        }
                        var_value.push(Some(value));
                    } else {
                        var_value.push(None);
                    }
                },
                _ => {
                    return None; // TODO Enviar error sintáctico
                }
            }

            // TODO funcion para detectar valores

            /*match x {
                Some ((token, TokenType::)) => var_name.push(token),
                _ => None, // TODO Enviar error sintáctico
            }*/

        }

        /*if let Some((_, TokenType::Semicolon)) = self.tokens.next() {
            Some(ASTNode::VariableDeclaration { var_type, var_name })
        } else {
            None
        }*/

        //println!("Self Tokens: {:?}", self.tokens.next());
        println!("Var_Type: {:?}; Var_Name: {:?}", var_type, var_name);

        //println!("Next {:?}", self.tokens.next());

        Some(ASTNode::VariableDeclaration {
            var_type,
            var_name,
            var_value,
        })
    }

    fn parse_variable_assignment(&mut self) -> Option<ASTNode> {
        let var_name = self.tokens.next()?.0;

        // Expect an assignment operator
        if node_dict::isOperator(self.tokens.next().unwrap().1) {
            let mut value = String::new();
            while let Some((token, token_type)) = self.tokens.peek() {
                if let TokenType::Semicolon = token_type {
                    break;
                } else {
                    value.push_str(token);
                    self.tokens.next();
                }
            }
            // Expect a semicolon
            self.tokens.next();
            Some(ASTNode::VariableAssignment { var_name, value })
        } else {
            None
        }
    }

    fn parse_while_loop(&mut self) -> Option<ASTNode> {
        self.tokens.next(); // Consume 'while'

        // Expect a condition in parentheses
        let mut condition = String::new();
        if let Some((_, TokenType::LParen)) = self.tokens.next() {
            while let Some((token, token_type)) = self.tokens.peek() {
                if let TokenType::RParen = token_type {
                    self.tokens.next(); // Consume closing parenthesis
                    break;
                } else {
                    condition.push_str(token);
                    self.tokens.next();
                }
            }
        }

        // Parse the body of the loop
        if let Some(ASTNode::Block(body)) = self.parse_block() {
            Some(ASTNode::WhileLoop { condition, body })
        } else {
            None
        }
    }

    fn parse_if_statement(&mut self) -> Option<ASTNode> {
        self.tokens.next(); // Consume 'if'

        // Expect a condition in parentheses
        let mut condition = String::new();
        if let Some((_, TokenType::LParen)) = self.tokens.next() {
            while let Some((token, token_type)) = self.tokens.peek() {
                if let TokenType::RParen = token_type {
                    self.tokens.next(); // Consume closing parenthesis
                    break;
                } else {
                    condition.push_str(token);
                    self.tokens.next();
                }
            }
        }

        // Parse the body of the if statement
        let body = if let Some(ASTNode::Block(body)) = self.parse_block() {
            body
        } else {
            return None;
        };

        // Check for an else statement
        let else_body = if let Some((else_token, _)) = self.tokens.peek() {
            if else_token == "else" {
                self.tokens.next(); // Consume 'else'
                if let Some(ASTNode::Block(else_body)) = self.parse_block() {
                    Some(else_body)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        Some(ASTNode::IfStatement {
            condition,
            body,
            else_body,
        })
    }

    fn parse_block(&mut self) -> Option<ASTNode> {
        if let Some((token, _)) = self.tokens.next() {
            if token != "{" {
                return None; // If the next token is not '{', return None
            }
        }

        let mut statements = Vec::new();
        while let Some((_token, token_type)) = self.tokens.peek() {
            if let TokenType::RBrace = token_type {
                self.tokens.next(); // Consume '}'
                break;
            }
            if let Some(statement) = self.parse_statement() {
                statements.push(statement);
            } else {
                self.tokens.next();
            }
        }

        Some(ASTNode::Block(statements))
    }

    fn parse_expression(&mut self) -> Option<ASTNode> {
        let mut expression = String::new();
        while let Some((token, token_type)) = self.tokens.peek() {
            if let TokenType::Semicolon = token_type {
                break;
            } else {
                expression.push_str(token);
                self.tokens.next();
            }
        }
        self.tokens.next(); // Consume semicolon
        Some(ASTNode::Expression(expression))
    }
}

// Process the code file
fn process_code(file_path: &str) {
    let mut in_multiline_comment = false;

    match read_file(file_path) {
        Ok(lines) => {
            let mut tokens = Vec::new();
            for (line_number, line) in lines.iter().enumerate() {
                let line_tokens = tokenize_line(&line, &mut in_multiline_comment);
                println!("Line {}: Tokens: {:?}", line_number + 1, line_tokens);
                tokens.extend(line_tokens);
            }

            let mut parser = Parser::new(tokens);
            let ast = parser.parse();

            println!("AST: {:?}", ast);
        }
        Err(e) => eprintln!("Error reading the file: {}", e),
    }
}

// Main function
fn main() {
    let file_path = "../prueba.txt"; // Modify this path to point to your test file
    process_code(file_path);
}
