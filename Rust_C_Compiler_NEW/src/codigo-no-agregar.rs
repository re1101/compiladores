use std::collections::HashMap;
use std::collections::VecDeque;

fn main() {
    let infix = "a + b * c - d / e";
    match infix_to_postfix(infix) {
        Ok(postfix) => println!("Postfix: {}", postfix),
        Err(e) => eprintln!("Error: {}", e),
    }
}

/// Converts an infix expression to postfix (Reverse Polish Notation).
fn infix_to_postfix(expression: &str) -> Result<String, String> {
    let precedence = build_precedence();
    let mut output = VecDeque::new();
    let mut operators = Vec::new();

    let tokens = tokenize(expression)?;

    for token in tokens {
        if is_operand(&token) {
            output.push_back(token);
        } else if is_operator(&token) {
            while let Some(op) = operators.last() {
                if is_operator(op)
                    && precedence[op] >= precedence[&token]
                    && token != "("
                {
                    output.push_back(operators.pop().unwrap());
                } else {
                    break;
                }
            }
            operators.push(token);
        } else if token == "(" {
            operators.push(token);
        } else if token == ")" {
            while let Some(op) = operators.pop() {
                if op == "(" {
                    break;
                }
                output.push_back(op);
            }
        } else {
            return Err(format!("Unknown token: {}", token));
        }
    }

    while let Some(op) = operators.pop() {
        if op == "(" {
            return Err("Mismatched parentheses".to_string());
        }
        output.push_back(op);
    }

    Ok(output.into_iter().collect::<Vec<_>>().join(" "))
}

/// Checks if a token is an operand (identifier or literal).
fn is_operand(token: &str) -> bool {
    token.chars().all(char::is_alphanumeric)
}

/// Checks if a token is an operator.
fn is_operator(token: &str) -> bool {
    let operators: [&str; 26] = [
        "+", "-", "*", "/", "%", "=", "==", "!=", ">", "<", ">=", "<=", "&&", "||", "!", "&", "|",
        "^", "~", "<<", ">>", "++", "--", "->", ".", ",", ";",
    ];
    operators.contains(&token)
}

/// Tokenizes the input expression into a vector of strings.
fn tokenize(expression: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut current_token = String::new();

    for ch in expression.chars() {
        if ch.is_whitespace() {
            if !current_token.is_empty() {
                tokens.push(current_token.clone());
                current_token.clear();
            }
        } else if is_operator(&ch.to_string()) || ch == '(' || ch == ')' {
            if !current_token.is_empty() {
                tokens.push(current_token.clone());
                current_token.clear();
            }
            tokens.push(ch.to_string());
        } else if ch.is_alphanumeric() || ch == '_' {
            current_token.push(ch);
        } else {
            return Err(format!("Invalid character in expression: {}", ch));
        }
    }

    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    Ok(tokens)
}

/// Builds a precedence table for C operators.
fn build_precedence() -> HashMap<String, u8> {
    let mut precedence = HashMap::new();
    precedence.insert("(".to_string(), 0);
    precedence.insert(")".to_string(), 0);
    precedence.insert("||".to_string(), 1);
    precedence.insert("&&".to_string(), 2);
    precedence.insert("|".to_string(), 3);
    precedence.insert("^".to_string(), 4);
    precedence.insert("&".to_string(), 5);
    precedence.insert("==".to_string(), 6);
    precedence.insert("!=".to_string(), 6);
    precedence.insert("<".to_string(), 7);
    precedence.insert("<=".to_string(), 7);
    precedence.insert(">".to_string(), 7);
    precedence.insert(">=".to_string(), 7);
    precedence.insert("<<".to_string(), 8);
    precedence.insert(">>".to_string(), 8);
    precedence.insert("+".to_string(), 9);
    precedence.insert("-".to_string(), 9);
    precedence.insert("*".to_string(), 10);
    precedence.insert("/".to_string(), 10);
    precedence.insert("%".to_string(), 10);
    precedence.insert("!".to_string(), 11);
    precedence.insert("~".to_string(), 11);
    precedence.insert("++".to_string(), 12);
    precedence.insert("--".to_string(), 12);
    precedence
}
