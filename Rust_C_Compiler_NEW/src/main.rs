use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

#[derive(Debug)]
enum TokenType {
    Identifier,
    ReservedWord,
    Semicolon,
    Colon,
    Operator,
    Parenthesis,
    SingleQuote,
    DoubleQuote,
    Backslash,
    Space,
    Other,
}

// List of C reserved words
const RESERVED_WORDS: &[&str] = &[
    "auto", "break", "case", "char", "const", "continue", "default", "do",
    "double", "else", "enum", "extern", "float", "for", "goto", "if", "inline",
    "int", "long", "register", "restrict", "return", "short", "signed", "sizeof",
    "static", "struct", "switch", "typedef", "union", "unsigned", "void", "volatile",
    "while", "_Alignas", "_Alignof", "_Atomic", "_Bool", "_Complex", "_Generic",
    "_Imaginary", "_Noreturn", "_Static_assert", "_Thread_local",
];

// List of operators and symbols
const OPERATORS_AND_SYMBOLS: &[&str] = &[
    "+", "-", "*", "/", "%", "=", "==", "!=", ">", "<", ">=", "<=", "&&", "||", "!",
    "&", "|", "^", "~", "<<", ">>", "++", "--", "->", ".", ";", ":", "(", ")", "{", "}",
];

fn is_identifier(token: &str) -> bool {
    // Define a regex pattern for C identifiers
    let identifier_re = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    identifier_re.is_match(token)
}

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
            current_token.push(ch);
            if ch == '"' && !escape {
                in_string = false;
                tokens.push((current_token.clone(), TokenType::DoubleQuote));
                current_token.clear();
            }
            escape = ch == '\\' && !escape;
            chars.next();
            continue;
        }

        if in_char {
            current_token.push(ch);
            if ch == '\'' && !escape {
                in_char = false;
                tokens.push((current_token.clone(), TokenType::SingleQuote));
                current_token.clear();
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
                    if RESERVED_WORDS.contains(&current_token.as_str()) {
                        tokens.push((current_token.clone(), TokenType::ReservedWord));
                    } else if is_identifier(&current_token) {
                        tokens.push((current_token.clone(), TokenType::Identifier));
                    } else {
                        tokens.push((current_token.clone(), TokenType::Other));
                    }
                    current_token.clear();
                }
                current_token.push(ch);
                in_string = true;
            }
            '\'' => {
                if !current_token.is_empty() {
                    if RESERVED_WORDS.contains(&current_token.as_str()) {
                        tokens.push((current_token.clone(), TokenType::ReservedWord));
                    } else if is_identifier(&current_token) {
                        tokens.push((current_token.clone(), TokenType::Identifier));
                    } else {
                        tokens.push((current_token.clone(), TokenType::Other));
                    }
                    current_token.clear();
                }
                current_token.push(ch);
                in_char = true;
            }
            _ if ch.is_whitespace() => {
                if !current_token.is_empty() {
                    if RESERVED_WORDS.contains(&current_token.as_str()) {
                        tokens.push((current_token.clone(), TokenType::ReservedWord));
                    } else if is_identifier(&current_token) {
                        tokens.push((current_token.clone(), TokenType::Identifier));
                    } else {
                        tokens.push((current_token.clone(), TokenType::Other));
                    }
                    current_token.clear();
                }
                tokens.push((" ".to_owned(), TokenType::Space)); // Add space as a token
            }
            _ if ch.is_alphanumeric() || ch == '_' => {
                current_token.push(ch);
            }
            _ => {
                if !current_token.is_empty() {
                    if RESERVED_WORDS.contains(&current_token.as_str()) {
                        tokens.push((current_token.clone(), TokenType::ReservedWord));
                    } else if is_identifier(&current_token) {
                        tokens.push((current_token.clone(), TokenType::Identifier));
                    } else {
                        tokens.push((current_token.clone(), TokenType::Other));
                    }
                    current_token.clear();
                }

                match ch {
                    ';' => tokens.push((";".to_owned(), TokenType::Semicolon)),
                    ':' => tokens.push((":".to_owned(), TokenType::Colon)),
                    '(' | ')' | '{' | '}' => tokens.push((ch.to_string(), TokenType::Parenthesis)),
                    '\\' => tokens.push(("\\".to_owned(), TokenType::Backslash)),
                    _ => {
                        let mut operator = String::new();
                        operator.push(ch);
                        chars.next();

                        while let Some(&next_ch) = chars.peek() {
                            operator.push(next_ch);
                            if !OPERATORS_AND_SYMBOLS.contains(&operator.as_str()) {
                                operator.pop();
                                break;
                            }
                            chars.next();
                        }

                        if OPERATORS_AND_SYMBOLS.contains(&operator.as_str()) {
                            tokens.push((operator.clone(), TokenType::Operator));
                        } else {
                            tokens.push((operator.clone(), TokenType::Other));
                        }
                        continue;
                    }
                }
            }
        }
        chars.next();
    }

    if !current_token.is_empty() {
        if RESERVED_WORDS.contains(&current_token.as_str()) {
            tokens.push((current_token.clone(), TokenType::ReservedWord));
        } else if is_identifier(&current_token) {
            tokens.push((current_token.clone(), TokenType::Identifier));
        } else {
            tokens.push((current_token.clone(), TokenType::Other));
        }
    }

    tokens
}

fn read_file(file_path: &str) -> io::Result<Vec<String>> {
    let path = Path::new(file_path);
    let file = File::open(&path)?;
    let lines = io::BufReader::new(file).lines();
    lines.collect()
}

fn process_code(file_path: &str) {
    let mut in_multiline_comment = false;

    match read_file(file_path) {
        Ok(lines) => {
            for (index, line) in lines.iter().enumerate() {
                let tokens = tokenize_line(line, &mut in_multiline_comment);
                for (token, token_type) in tokens {
                    match token_type {
                        TokenType::Identifier => println!("Line {}: '{}' is an Identifier", index + 1, token),
                        TokenType::ReservedWord => println!("Line {}: '{}' is a Reserved Word", index + 1, token),
                        TokenType::Semicolon => println!("Line {}: '{}' is a Semicolon", index + 1, token),
                        TokenType::Colon => println!("Line {}: '{}' is a Colon", index + 1, token),
                        TokenType::Operator => println!("Line {}: '{}' is an Operator", index + 1, token),
                        TokenType::Parenthesis => println!("Line {}: '{}' is a Parenthesis", index + 1, token),
                        TokenType::SingleQuote => println!("Line {}: '{}' is a Char", index + 1, token),
                        TokenType::DoubleQuote => println!("Line {}: '{}' is a String", index + 1, token),
                        TokenType::Backslash => println!("Line {}: '{}' is a Backslash", index + 1, token),
                        TokenType::Space => println!("Line {}: ' ' is a Space", index + 1),
                        TokenType::Other => println!("Line {}: '{}' is not an Identifier", index + 1, token),
                    }
                }
            }
        }
        Err(e) => eprintln!("Error reading the file: {}", e),
    }
}

fn main() {
    let file_path = "../prueba.txt";  // Modify this path to point to your test file
    process_code(file_path);
}