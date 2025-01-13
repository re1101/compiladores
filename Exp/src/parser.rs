use crate::token::Token;

#[derive(Debug, Clone)]
pub enum ASTNode {
    Function {
        name: String,
        body: Vec<ASTNode>,
    },
    Return {
        value: Box<ASTNode>,
    },
    VariableDeclaration {
        data_type: String,
        name: String,
        value: Option<Box<ASTNode>>,
    },
    Expression(String),
    Literal(String),
    CharLiteral(char),
    If {
        condition: Box<ASTNode>,
        body: Vec<ASTNode>,
        else_body: Option<Vec<ASTNode>>,
    },
    While {
        condition: Box<ASTNode>,
        body: Vec<ASTNode>,
    },
    Assignment {
        name: String,
        value: Box<ASTNode>,
    },
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    errors: Vec<String>,
    ast: Vec<ASTNode>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, position: 0, errors: Vec::new(), ast: Vec::new() }
    }

    pub fn parse(&mut self) -> Option<Vec<ASTNode>> {
        while self.position < self.tokens.len() {
            if let Some(node) = self.parse_node() {
                self.ast.push(node);
            } else {
                self.position += 1;
            }
        }

        if !self.errors.is_empty() {
            for error in &self.errors {
                println!("{}", error);
            }
            println!("AST: None");
            println!("Failed to parse input");
            None
        } else {
            println!("AST: {:?}", self.ast);
            Some(self.ast.clone())
        }
    }

    fn parse_node(&mut self) -> Option<ASTNode> {
        if let Some(node) = self.parse_function() {
            return Some(node);
        } else if let Some(node) = self.parse_variable_declaration() {
            return Some(node);
        } else if let Some(node) = self.parse_statement() {
            return Some(node);
        }
        None
    }

    fn parse_function(&mut self) -> Option<ASTNode> {
        let start_pos = self.position;

        if self.match_data_type() {
            let identifier_name = if let Some(Token::Identifier(name)) = self.tokens.get(self.position) {
                name.clone()
            } else {
                self.position = start_pos;
                return None;
            };

            self.position += 1;

            if self.match_symbol('(') {
                while self.position < self.tokens.len() && self.tokens.get(self.position) != Some(&Token::Symbol(')')) {
                    self.position += 1;
                }

                if self.match_symbol(')') {
                    if self.match_symbol('{') {
                        let body = self.parse_body();

                        if self.match_symbol('}') {
                            return Some(ASTNode::Function {
                                name: identifier_name,
                                body,
                            });
                        } else {
                            self.position = start_pos;
                            return None;
                        }
                    } else {
                        self.position = start_pos;
                        return None;
                    }
                } else {
                    self.position = start_pos;
                    return None;
                }
            } else {
                self.position = start_pos;
                None
            }
        } else {
            None
        }
    }

    fn parse_body(&mut self) -> Vec<ASTNode> {
        let mut body = Vec::new();
        while self.position < self.tokens.len() {
            if self.tokens.get(self.position) == Some(&Token::Symbol('}')) {
                break;
            }

            if let Some(statement) = self.parse_statement() {
                body.push(statement);
            } else {
                self.position += 1;
            }
        }
        if self.position >= self.tokens.len() || self.tokens.get(self.position) != Some(&Token::Symbol('}')) {
            let line = self.get_current_line();
            self.errors.push(format!("Error: Unmatched opening brace, missing '}}' at line {}", line));
        }
        body
    }

    fn parse_statement(&mut self) -> Option<ASTNode> {
        if let Some(return_statement) = self.parse_return() {
            return Some(return_statement);
        } else if let Some(if_statement) = self.parse_if() {
            return Some(if_statement);
        } else if let Some(while_statement) = self.parse_while() {
            return Some(while_statement);
        } else if let Some(assignment) = self.parse_assignment() {
            return Some(assignment);
        } else if let Some(expression) = self.parse_expression() {
            return Some(expression);
        } else {
            let line = self.get_current_line();
            self.errors.push(format!("Error: Unexpected token at line {}", line));
            self.position += 1;
            None
        }
    }

    fn parse_return(&mut self) -> Option<ASTNode> {
        if let Some(Token::ControlStructure(keyword)) = self.tokens.get(self.position) {
            if keyword == "return" {
                self.position += 1;
                if let Some(value) = self.parse_literal_or_identifier() {
                    if self.match_symbol(';') {
                        return Some(ASTNode::Return {
                            value: Box::new(value),
                        });
                    } else {
                        let line = self.get_current_line();
                        self.errors.push(format!("Error: Expected ';' after return value at line {}", line));
                    }
                }
            }
        }
        None
    }

    fn parse_if(&mut self) -> Option<ASTNode> {
        if self.match_control_structure("if") && self.match_symbol('(') {
            if let Some(condition) = self.parse_expression() {
                if self.match_symbol(')') && self.match_symbol('{') {
                    let body = self.parse_body();
                    if self.match_symbol('}') {
                        let else_body = if self.match_control_structure("else") && self.match_symbol('{') {
                            let else_body = self.parse_body();
                            if self.match_symbol('}') {
                                Some(else_body)
                            } else {
                                let line = self.get_current_line();
                                self.errors.push(format!("Error: Unmatched opening brace in else block at line {}", line));
                                None
                            }
                        } else {
                            None
                        };
                        return Some(ASTNode::If {
                            condition: Box::new(condition),
                            body,
                            else_body,
                        });
                    } else {
                        let line = self.get_current_line();
                        self.errors.push(format!("Error: Unmatched opening brace in if block at line {}", line));
                        return None;
                    }
                } else {
                    let line = self.get_current_line();
                    self.errors.push(format!("Error: Expected '{{' after if condition at line {}", line));
                    return None;
                }
            } else {
                let line = self.get_current_line();
                self.errors.push(format!("Error: Expected ')' after if condition at line {}", line));
                return None;
            }
        }
        None
    }

    fn parse_while(&mut self) -> Option<ASTNode> {
        if self.match_control_structure("while") && self.match_symbol('(') {
            if let Some(condition) = self.parse_expression() {
                if self.match_symbol(')') && self.match_symbol('{') {
                    let body = self.parse_body();
                    if self.match_symbol('}') {
                        return Some(ASTNode::While {
                            condition: Box::new(condition),
                            body,
                        });
                    } else {
                        let line = self.get_current_line();
                        self.errors.push(format!("Error: Unmatched opening brace in while block at line {}", line));
                        return None;
                    }
                } else {
                    let line = self.get_current_line();
                    self.errors.push(format!("Error: Expected '{{' after while condition at line {}", line));
                    return None;
                }
            } else {
                let line = self.get_current_line();
                self.errors.push(format!("Error: Expected ')' after while condition at line {}", line));
                return None;
            }
        }
        None
    }

    fn parse_variable_declaration(&mut self) -> Option<ASTNode> {
        let start_pos = self.position;
        if let Some(Token::DataType(data_type)) = self.tokens.get(self.position) {
            let data_type = data_type.clone();
            self.position += 1;
            if let Some(Token::Identifier(name)) = self.tokens.get(self.position) {
                let name = name.clone();
                self.position += 1;
                let value = if self.match_symbol('=') {
                    self.parse_literal_or_identifier().map(Box::new)
                } else {
                    None
                };
                if self.match_symbol(';') {
                    return Some(ASTNode::VariableDeclaration {
                        data_type,
                        name,
                        value,
                    });
                } else {
                    let line = self.get_current_line();
                    self.errors.push(format!("Error: Expected ';' after variable declaration at line {}", line));
                    self.position = start_pos;
                    return None;
                }
            }
        }
        None
    }

    fn parse_assignment(&mut self) -> Option<ASTNode> {
        if let Some(Token::Identifier(name)) = self.tokens.get(self.position) {
            let name = name.clone();
            self.position += 1;
            if self.match_symbol('=') {
                if let Some(value) = self.parse_literal_or_identifier() {
                    if self.match_symbol(';') {
                        return Some(ASTNode::Assignment {
                            name,
                            value: Box::new(value),
                        });
                    } else {
                        let line = self.get_current_line();
                        self.errors.push(format!("Error: Expected ';' after assignment at line {}", line));
                        return None;
                    }
                }
            }
        }
        None
    }

    fn parse_expression(&mut self) -> Option<ASTNode> {
        if let Some(identifier) = self.parse_identifier() {
            if self.match_symbol('(') {
                while !self.match_symbol(')') && self.position < self.tokens.len() {
                    self.position += 1;
                }
                if self.match_symbol(')') && self.match_symbol(';') {
                    return Some(ASTNode::Expression(identifier));
                } else {
                    let line = self.get_current_line();
                    self.errors.push(format!("Error: Expected ';' after function call at line {}", line));
                    return None;
                }
            } else if self.match_symbol(';') {
                return Some(ASTNode::Expression(identifier));
            } else {
                let line = self.get_current_line();
                self.errors.push(format!("Error: Expected ';' after expression at line {}", line));
                return None;
            }
        }
        None
    }

    fn parse_literal_or_identifier(&mut self) -> Option<ASTNode> {
        if let Some(token) = self.tokens.get(self.position) {
            match token {
                Token::StringLiteral(value) => {
                    self.position += 1;
                    Some(ASTNode::Literal(value.clone()))
                }
                Token::CharLiteral(value) => {
                    self.position += 1;
                    Some(ASTNode::CharLiteral(*value))
                }
                Token::Number(value) => {
                    self.position += 1;
                    Some(ASTNode::Literal(value.clone()))
                }
                Token::Identifier(name) => {
                    self.position += 1;
                    Some(ASTNode::Expression(name.clone()))
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn match_data_type(&mut self) -> bool {
        if let Some(Token::DataType(_)) = self.tokens.get(self.position) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    fn match_control_structure(&mut self, keyword: &str) -> bool {
        if let Some(Token::ControlStructure(id)) = self.tokens.get(self.position) {
            if id == keyword {
                self.position += 1;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn match_identifier(&mut self, identifier: &str) -> bool {
        if let Some(Token::Identifier(id)) = self.tokens.get(self.position) {
            if id == identifier {
                self.position += 1;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn match_symbol(&mut self, symbol: char) -> bool {
        if let Some(Token::Symbol(sym)) = self.tokens.get(self.position) {
            if *sym == symbol {
                self.position += 1;
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn parse_identifier(&mut self) -> Option<String> {
        if let Some(Token::Identifier(id)) = self.tokens.get(self.position) {
            self.position += 1;
            Some(id.clone())
        } else {
            None
        }
    }

    fn get_current_line(&self) -> usize {
        let mut line = 1;
        for i in 0..self.position {
            if let Some(token) = self.tokens.get(i) {
                if let Token::Symbol(sym) = token {
                    if *sym == '\n' {
                        line += 1;
                    }
                }
            }
        }
        line
    }
}