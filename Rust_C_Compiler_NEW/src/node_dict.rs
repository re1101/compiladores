// Token types
#[derive(Debug)]
pub enum TokenType {
    // Variables
    Identifier,
    
    // Data types
    Integer, String, Char, Float, Double, Void,

    // Reserved Words
    If, Print, Else,
    While, For, And, Or,
    True, False, Assign,

    // Symbols
    Plus, Minus, Mult, Division,
    Equals, More, MoreEquals, Less, LessEquals,
    LParen, RParen, LBrace, RBrace, LBrack, RBrack, Semicolon,
    Comment, MultilineComment, Backslash, Comma, Colon
}

// List of C reserved words
pub const RESERVED_WORDS: &[&str] = &[
    "auto",
    "break",
    "case",
    "char",
    "const",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extern",
    "float",
    "for",
    "goto",
    "if",
    "inline",
    "int",
    "long",
    "register",
    "restrict",
    "return",
    "short",
    "signed",
    "sizeof",
    "static",
    "struct",
    "switch",
    "typedef",
    "union",
    "unsigned",
    "void",
    "volatile",
    "while",
    "_Alignas",
    "_Alignof",
    "_Atomic",
    "_Bool",
    "_Complex",
    "_Generic",
    "_Imaginary",
    "_Noreturn",
    "_Static_assert",
    "_Thread_local",
];

// List of operators and symbols
pub const OPERATORS_AND_SYMBOLS: &[&str] = &[
    "+", "-", "*", "/", "%", "=", "==", "!=", ">", "<", ">=", "<=", "&&", "||", "!", "&", "|", "^",
    "~", "<<", ">>", "++", "--", "->", ".", ",", ";", ":", "(", ")", "{", "}",
];

pub fn isOperator(tokenType: TokenType) -> bool {
    match tokenType {
        TokenType::Plus
        | TokenType::Minus
        | TokenType::Mult
        | TokenType::Division
        | TokenType::Assign
        | TokenType::Equals
        | TokenType::More
        | TokenType::MoreEquals
        | TokenType::Less
        | TokenType::LessEquals
        | TokenType::And
        | TokenType::Or => true,
        _ => false,
    }
}

pub fn isNumber(tokenType: TokenType) -> bool {
    match tokenType {
        TokenType::Integer
        | TokenType::Float => true,
        _ => false,
    }
}

pub fn isDataType(tokenType: TokenType) -> bool {
    match tokenType {
        TokenType::String
        | TokenType::Identifier
        | TokenType::Integer
        | TokenType::True
        | TokenType::False => true,
        _ => false,
    }
}

pub fn isReserved(tokenType: TokenType) -> bool {
    match tokenType {
        /*TokenType::Var
        | TokenType::Set
        | */TokenType::If
        | TokenType::Print
        | TokenType::True
        | TokenType::False
        | TokenType::Else
        | TokenType::While
        | TokenType::For => true,
        _ => false,
    }
}

pub fn isControl(tokenType: TokenType) -> bool {
    match tokenType {
        TokenType::If
        | TokenType::Else
        | TokenType::While
        | TokenType::For => true,
        _ => false,
    }
}
