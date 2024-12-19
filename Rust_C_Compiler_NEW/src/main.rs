use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Debug, Clone)]
struct SymbolTable {
    variables: HashMap<String, String>,
    functions: HashMap<String, (String, Vec<String>)>,
}

impl SymbolTable {
    fn new() -> Self {
        SymbolTable {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    fn add_variable(&mut self, name: &str, var_type: &str) {
        self.variables.insert(name.to_string(), var_type.to_string());
    }

    fn add_function(&mut self, name: &str, return_type: &str, param_types: Vec<String>) {
        self.functions.insert(name.to_string(), (return_type.to_string(), param_types));
    }

    fn is_function_defined(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    fn get_function_signature(&self, name: &str) -> Option<&(String, Vec<String>)> {
        self.functions.get(name)
    }

    fn is_variable_defined(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    fn get_variable_type(&self, name: &str) -> Option<&String> {
        self.variables.get(name)
    }
}

#[derive(Debug)]
enum ASTNode {
    VariableDeclaration { var_type: String, var_name: String },
    FunctionDefinition { return_type: String, func_name: String, parameters: String, body: Vec<ASTNode> },
    ControlStructure { control_type: String, condition: String, body: Vec<ASTNode> },
    WhileLoop { condition: String, body: Vec<ASTNode> },
    ReturnStatement { value: String },
    Assignment { var_name: String, value: String },
    FunctionCall { func_name: String, arguments: Vec<String> },
    FunctionClosure,
    Unknown { content: String },
}

struct AST {
    nodes: Vec<ASTNode>,
}

impl AST {
    fn new() -> Self {
        AST { nodes: Vec::new() }
    }

    fn add_node(&mut self, node: ASTNode) {
        self.nodes.push(node);
    }

    fn print(&self) {
        for node in &self.nodes {
            println!("{:?}", node);
        }
    }
}

fn parse_block(
    lines: &[String],
    sym_table: &mut SymbolTable,
    index: &mut usize,
    depth: usize,
    max_depth: usize,
) -> (Vec<ASTNode>, bool) {
    let mut body = Vec::new();
    let mut local_sym_table = sym_table.clone();
    let mut in_multiline_comment = false;
    let mut has_return = false;

    if depth > max_depth {
        println!("Error: Maximum recursion depth exceeded");
        return (body, has_return);
    }

    while *index < lines.len() {
        let line = lines[*index].trim();

        if in_multiline_comment {
            if line.contains("*/") {
                in_multiline_comment = false;
            }
            *index += 1;
            continue;
        }

        if line.is_empty() || line.starts_with("//") {
            *index += 1;
            continue;
        }

        if line.contains("/*") {
            in_multiline_comment = true;
            *index += 1;
            continue;
        }

        if line == "}" {
            *index += 1;
            break;
        }

        match check_syntax(line, &mut local_sym_table) {
            Ok(node) => match node {
                ASTNode::FunctionDefinition {
                    return_type,
                    func_name,
                    parameters,
                    ..
                } => {
                    *index += 1;
                    let (func_body, func_has_return) = parse_block(lines, &mut local_sym_table, index, depth + 1, max_depth);
                    if return_type != "void" && !func_has_return {
                        println!("Error: Function '{}' missing return statement", func_name);
                    }
                    body.push(ASTNode::FunctionDefinition {
                        return_type,
                        func_name,
                        parameters,
                        body: func_body,
                    });
                }
                ASTNode::ControlStructure {
                    control_type,
                    condition,
                    ..
                } => {
                    *index += 1;
                    let (control_body, control_has_return) = parse_block(lines, &mut local_sym_table, index, depth + 1, max_depth);
                    body.push(ASTNode::ControlStructure {
                        control_type,
                        condition,
                        body: control_body,
                    });
                    has_return |= control_has_return;
                }
                ASTNode::WhileLoop { .. } => {
                    *index += 1;
                    let (while_body, while_has_return) = parse_block(lines, &mut local_sym_table, index, depth + 1, max_depth);
                    if let Some(captures) = while_re().captures(&lines[*index]) {
                        let condition = captures.get(1).unwrap().as_str().to_string();
                        body.push(ASTNode::WhileLoop {
                            condition,
                            body: while_body,
                        });
                        *index += 1;
                    } else {
                        println!("Error: Malformed while condition");
                        break;
                    }
                    has_return |= while_has_return;
                }
                ASTNode::ReturnStatement { .. } => {
                    has_return = true;
                    body.push(node);
                }
                _ => body.push(node),
            },
            Err(err) => println!("Line {}: {}", *index + 1, err),
        }

        *index += 1;
    }

    sym_table.variables.extend(local_sym_table.variables);
    (body, has_return)
}

fn extract_variables_and_calls(condition: &str) -> (Vec<String>, Vec<String>) {
    let re = Regex::new(r"\b[a-zA-Z_]\w*\b").unwrap();
    let mut variables = Vec::new();
    let mut calls = Vec::new();

    for mat in re.find_iter(condition) {
        let name = mat.as_str().to_string();
        if condition.contains(&format!("{}(", name)) {
            calls.push(name);
        } else {
            variables.push(name);
        }
    }

    (variables, calls)
}

fn extract_arguments(args_str: &str) -> Vec<String> {
    args_str.split(',').map(|s| s.trim().to_string()).collect()
}

fn check_argument_type(arg: &str, expected_type: &str, sym_table: &SymbolTable) -> Result<(), String> {
    if arg.starts_with('\'') && arg.ends_with('\'') {
        if expected_type != "char" {
            return Err(format!("Expected type '{}' but got 'char'", expected_type));
        }
    } else if let Some(var_type) = sym_table.get_variable_type(arg) {
        if var_type != expected_type {
            return Err(format!("Expected type '{}' but variable '{}' is of type '{}'", expected_type, arg, var_type));
        }
    } else if let Ok(_) = arg.parse::<i32>() {
        if expected_type != "int" {
            return Err(format!("Expected type '{}' but got 'int'", expected_type));
        }
    } else if let Ok(_) = arg.parse::<f32>() {
        if expected_type != "float" {
            return Err(format!("Expected type '{}' but got 'float'", expected_type));
        }
    } else {
        return Err(format!("Undefined variable '{}'", arg));
    }
    Ok(())
}

fn check_syntax(line: &str, sym_table: &mut SymbolTable) -> Result<ASTNode, String> {
    let var_decl_re = Regex::new(r"^\s*(int|float|char|void)\s+((\w+\s*(=\s*[^,;]+)?\s*,\s*)*\w+\s*(=\s*[^,;]+)?)\s*;\s*$").unwrap();
    let func_def_re = Regex::new(r"^\s*(int|float|void|char)\s+(\w+)\s*\(([^)]*)\)\s*\{?\s*$").unwrap();
    let control_re = Regex::new(r"^\s*(if|else|while|for|switch)\s*\((.*)\)\s*\{?\s*$").unwrap();
    let return_re = Regex::new(r"^\s*return\s+(.+);\s*$").unwrap();
    let assignment_re = Regex::new(r"^\s*(\w+)\s*=\s*(.+);\s*$").unwrap();
    let func_call_re = Regex::new(r"^\s*(\w+)\s*\(([^)]*)\)\s*;\s*$").unwrap();
    let char_literal_re = Regex::new(r"^'.'$").unwrap();

    if var_decl_re.is_match(line) {
        let captures = var_decl_re.captures(line).unwrap();
        let var_type = captures.get(1).unwrap().as_str().to_string();
        let var_list = captures.get(2).unwrap().as_str();

        for var_decl in var_list.split(',') {
            let parts: Vec<&str> = var_decl.trim().split('=').collect();
            let var_name = parts[0].trim().to_string();
            if parts.len() > 1 {
                let value = parts[1].trim();
                if func_call_re.is_match(value) {
                    let func_captures = func_call_re.captures(value).unwrap();
                    let func_name = func_captures.get(1).unwrap().as_str();
                    let args_str = func_captures.get(2).unwrap().as_str();
                    let args = extract_arguments(args_str);

                    if let Some((_, param_types)) = sym_table.get_function_signature(func_name) {
                        if args.len() != param_types.len() {
                            return Err(format!("Error: Function '{}' called with incorrect number of arguments", func_name));
                        }
                        for (arg, param_type) in args.iter().zip(param_types.iter()) {
                            check_argument_type(arg, param_type, sym_table)?;
                        }
                    } else {
                        return Err(format!("Error: Undefined function call '{}'", func_name));
                    }
                }
            }
            sym_table.add_variable(&var_name, &var_type);
        }

        Ok(ASTNode::VariableDeclaration {
            var_type,
            var_name: var_list.to_string(),
        })
    } else if func_def_re.is_match(line) {
        let captures = func_def_re.captures(line).unwrap();
        let return_type = captures.get(1).unwrap().as_str().to_string();
        let func_name = captures.get(2).unwrap().as_str().to_string();
        let parameters = captures.get(3).unwrap().as_str().to_string();
        let param_types = parameters.split(',')
                                    .map(|param| param.trim().split_whitespace().next().unwrap_or("").to_string())
                                    .collect();

        // Add function parameters to the symbol table
        for param in parameters.split(',') {
            let parts: Vec<&str> = param.trim().split_whitespace().collect();
            if parts.len() == 2 {
                let param_type = parts[0].to_string();
                let param_name = parts[1].to_string();
                sym_table.add_variable(&param_name, &param_type);
            }
        }

        sym_table.add_function(&func_name, &return_type, param_types);
        Ok(ASTNode::FunctionDefinition {
            return_type,
            func_name,
            parameters,
            body: Vec::new(),
        })
    } else if control_re.is_match(line) {
        let captures = control_re.captures(line).unwrap();
        let control_type = captures.get(1).unwrap().as_str().to_string();
        let condition = captures.get(2).unwrap().as_str().to_string();

        let (variables, calls) = extract_variables_and_calls(&condition);
        for var in variables {
            if !sym_table.is_variable_defined(&var) && !sym_table.is_function_defined(&var) && !char_literal_re.is_match(&var) {
                return Err(format!("Error: Undefined variable in condition '{}'", var));
            }
        }
        for call in calls {
            if !sym_table.is_function_defined(&call) {
                return Err(format!("Error: Undefined function call in condition '{}'", call));
            }
        }

        Ok(ASTNode::ControlStructure {
            control_type,
            condition,
            body: Vec::new(),
        })
    } else if while_re().is_match(line) {
        let captures = while_re().captures(line).unwrap();
        let condition = captures.get(1).unwrap().as_str().to_string();

        Ok(ASTNode::WhileLoop {
            condition,
            body: Vec::new(),
        })
    } else if return_re.is_match(line) {
        let captures = return_re.captures(line).unwrap();
        let value = captures.get(1).unwrap().as_str().to_string();

        Ok(ASTNode::ReturnStatement { value })
    } else if assignment_re.is_match(line) {
        let captures = assignment_re.captures(line).unwrap();
        let var_name = captures.get(1).unwrap().as_str().to_string();
        let value = captures.get(2).unwrap().as_str().to_string();

        // Check if the value is a function call
        if func_call_re.is_match(&value) {
            let func_captures = func_call_re.captures(&value).unwrap();
            let func_name = func_captures.get(1).unwrap().as_str();
            let args_str = func_captures.get(2).unwrap().as_str();
            let args = extract_arguments(args_str);

            if let Some((_, param_types)) = sym_table.get_function_signature(func_name) {
                if args.len() != param_types.len() {
                    return Err(format!("Error: Function '{}' called with incorrect number of arguments", func_name));
                }
                for (arg, param_type) in args.iter().zip(param_types.iter()) {
                    check_argument_type(arg, param_type, sym_table)?;
                }
            } else {
                return Err(format!("Error: Undefined function call '{}'", func_name));
            }

            Ok(ASTNode::FunctionCall {
                func_name: func_name.to_string(),
                arguments: args,
            })
        } else {
            if !sym_table.is_variable_defined(&var_name) {
                return Err(format!("Error: Undefined variable '{}'", var_name));
            }

            Ok(ASTNode::Assignment { var_name, value })
        }
    } else {
        Err(format!("Syntax error: {}", line))
    }
}

fn while_re() -> Regex {
    Regex::new(r"^\s*while\s*\((.*)\)\s*\{?\s*$").unwrap()
}

fn read_file(file_path: &str) -> io::Result<Vec<String>> {
    let path = Path::new(file_path);
    let file = File::open(&path)?;
    let lines = io::BufReader::new(file).lines();
    lines.collect()
}

fn process_code(file_path: &str) {
    let mut sym_table = SymbolTable::new();
    let mut ast = AST::new();
    let mut in_multiline_comment = false;
    let max_depth = 100;

    match read_file(file_path) {
        Ok(lines) => {
            let mut index = 0;
            while index < lines.len() {
                let line = lines[index].trim();

                if in_multiline_comment {
                    if line.contains("*/") {
                        in_multiline_comment = false;
                    }
                    index += 1;
                    continue;
                }

                if line.is_empty() || line.starts_with("//") {
                    index += 1;
                    continue;
                }

                if line.contains("/*") {
                    in_multiline_comment = true;
                    index += 1;
                    continue;
                }

                match check_syntax(line, &mut sym_table) {
                    Ok(node) => match node {
                        ASTNode::FunctionDefinition {
                            return_type,
                            func_name,
                            parameters,
                            ..
                        } => {
                            index += 1;
                            let (body, has_return) = parse_block(&lines, &mut sym_table, &mut index, 1, max_depth);
                            if return_type != "void" && !has_return {
                                println!("Error: Function '{}' missing return statement", func_name);
                            }
                            ast.add_node(ASTNode::FunctionDefinition {
                                return_type,
                                func_name,
                                parameters,
                                body,
                            });
                        }
                        ASTNode::ControlStructure {
                            control_type,
                            condition,
                            ..
                        } => {
                            index += 1;
                            let (body, _) = parse_block(&lines, &mut sym_table, &mut index, 1, max_depth);
                            ast.add_node(ASTNode::ControlStructure {
                                control_type,
                                condition,
                                body,
                            });
                        }
                        ASTNode::WhileLoop { .. } => {
                            index += 1;
                            let (body, _) = parse_block(&lines, &mut sym_table, &mut index, 1, max_depth);
                            let condition = if let Some(captures) = while_re().captures(&lines[index]) {
                                captures.get(1).unwrap().as_str().to_string()
                            } else {
                                println!("Error: Malformed while condition");
                                break;
                            };
                            ast.add_node(ASTNode::WhileLoop { condition, body });
                            index += 1;
                        }
                        _ => ast.add_node(node),
                    },
                    Err(err) => println!("Line {}: {}", index + 1, err),
                }

                index += 1;
            }
            println!("AST:");
            ast.print();
        }
        Err(e) => eprintln!("Error al leer el archivo: {}", e),
    }
}

fn main() {
    let file_path = "../prueba.txt";
    process_code(file_path);
}