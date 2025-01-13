use crate::parser::ASTNode;

pub struct SemanticAnalyzer;

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer
    }

    pub fn analyze(&mut self, nodes: &Vec<ASTNode>) -> Result<(), String> {
        for node in nodes {
            self.analyze_node(node)?;
        }
        Ok(())
    }

    fn analyze_node(&mut self, node: &ASTNode) -> Result<(), String> {
        match node {
            ASTNode::Function { name, body } => {
                println!("Analyzing function: {}", name);
                self.analyze(body)?;
            }
            ASTNode::Return { value } => {
                println!("Analyzing return statement");
                self.analyze_node(value)?;
            }
            ASTNode::VariableDeclaration { data_type, name, value } => {
                println!("Analyzing variable declaration: {} {}", data_type, name);
                if let Some(value) = value {
                    self.analyze_node(value)?;
                }
            }
            ASTNode::Expression(expr) => {
                println!("Analyzing expression: {}", expr);
            }
            ASTNode::Literal(lit) => {
                println!("Analyzing literal: {}", lit);
            }
            ASTNode::CharLiteral(chr) => {
                println!("Analyzing char literal: {}", chr);
            }
            ASTNode::If { condition, body, else_body } => {
                println!("Analyzing if statement");
                self.analyze_node(condition)?;
                self.analyze(body)?;
                if let Some(else_body) = else_body {
                    self.analyze(else_body)?;
                }
            }
            ASTNode::While { condition, body } => {
                println!("Analyzing while statement");
                self.analyze_node(condition)?;
                self.analyze(body)?;
            }
            ASTNode::Assignment { name, value } => {
                println!("Analyzing assignment: {}", name);
                self.analyze_node(value)?;
            }
        }
        Ok(())
    }
}