#![allow(non_snake_case)]

use indexmap::indexmap;
use regex::Regex;
use std::{
    collections::BTreeMap,
    error::Error,
    fmt,
    fs::File,
    io::{self, BufRead},
    path::Path,
};

#[allow(unused_imports)]
use ferris_says::say;

#[macro_use]
extern crate ferris_print;
extern crate ferris_says;

#[derive(Debug)]
enum CompilerError {
    OpenError(std::io::Error),
    RegexError(regex::Error),
    InvalidSyntax(u16),
    InvalidToken(String, u16),
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompilerError::OpenError(err) => write!(f, "Captured Underlying Error: {}", err),
            CompilerError::RegexError(err) => write!(f, "Captured Underlying Regex Error: {}", err),
            CompilerError::InvalidSyntax(line) => write!(f, "Invalid syntax at line: {}", line),
            CompilerError::InvalidToken(token, line) => {
                write!(f, "Invalid token \"{}\" at line: {}", token, line)
            }
        }
    }
}

impl Error for CompilerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CompilerError::OpenError(err) => Some(err),
            _ => None,
        }
    }
}

fn main() -> Result<(), CompilerError> {
    let path = "../prueba.txt"; // Ubicacion del archivo de texto
    let _file = match File::open(path) {
        Ok(file) => file,
        Err(e) => return Err(CompilerError::OpenError(e)),
    };

    let mut linesBuffer = String::new();

    // Leer el archivo línea por línea
    if let Ok(lines) = read_lines(path) {
        for line in lines {
            if let Ok(line) = line {
                if is_line_commented(&line) {
                    continue;
                }
                linesBuffer.push_str(&line); // Agrega la línea al buffer
            }
        }

        // Expresión regular para capturar palabras con sus delimitadores
        let re = Regex::new(r"[^\s{};]+(?:\s[^\s{};]+)*[{};]?|[{};]").unwrap();
        // Buscar coincidencias en la cadena
        let logicalLines: Vec<&str> = re.find_iter(&linesBuffer).map(|m| m.as_str()).collect();

        let mut currentLine: u16 = 0;

        let mut var_table: BTreeMap<String, String> = BTreeMap::new();
        let mut fn_table: BTreeMap<String, String> = BTreeMap::new();

        for logicalLine in logicalLines {
            let logicalLine = &multiline_comment(logicalLine);
            let logicalLine = &string_char_handling(logicalLine);
            let logicalLine = &logicalLine.replace("&&", " TEMPAND");
            let logicalLine = &logicalLine.replace("||", " TEMPOR");
            let logicalLine = &logicalLine.replace("==", " TEMPEQUALS");
            let logicalLine = &logicalLine.replace(">=", " TEMPEQG");
            let logicalLine = &logicalLine.replace("<=", " TEMPEQL");
            let logicalLine = &logicalLine.replace("!=", " TEMPNOT");
            let logicalLine = &logicalLine.replace("<>", " TEMPDIFF");

            // Expresión regular para capturar palabras, simbolos y delimitadores
            let reT = Regex::new(r"\w+|[^\w\s]").unwrap();
            // Buscar coincidencias en la cadena
            let tempTokens: Vec<&str> = reT.find_iter(&logicalLine).map(|m| m.as_str()).collect();

            let mut tokens: Vec<&str> = Vec::new();

            for tempToken in tempTokens {
                tokens.push(match tempToken {
                    "TEMPAND" => "&&",
                    "TEMPOR" => "||",
                    "TEMPEQUALS" => "==",
                    "TEMPEQG" => ">=",
                    "TEMPEQL" => "<=",
                    "TEMPNOT" => "!=",
                    "TEMPDIFF" => "<>",
                    _ => tempToken,
                });
            }

            let mut families: Vec<&str> = Vec::new();

            for token in tokens {
                families.push(match replace_tokens(token, currentLine) {
                    Ok(val) => val,
                    Err(e) => return Err(e),
                });
            }

            let newLine: String = families.into_iter().collect();

            match check_syntax(&newLine, currentLine) {
                Ok(val) => println!("{}", val),
                Err(e) => return Err(e),
            };

            currentLine += 1;
        }

        ferrisprint!("¡Compilado Exitoso!");
    }

    Ok(())
}

// Función para leer las líneas del archivo
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn is_line_commented(line: &str) -> bool {
    let SINGLE_LINE_COMMENT_REGEX = Regex::new( r"\/\/.*"); // Soporte para comentarios unilinea
    
    let mut res: bool = false;

    match SINGLE_LINE_COMMENT_REGEX{
        Ok(ref regex) => if regex.is_match(line) {res = true;},
        _ => {}
    }

    res
}

fn multiline_comment(line: &str) -> String{
    let MULTI_LINE_COMMENT_REGEX = Regex::new( r"/\*.*?\*/").unwrap(); // Soporte para comentarios multilinea
    
    let res: String = MULTI_LINE_COMMENT_REGEX.replace_all(line, "").to_string();

    res
}

fn string_char_handling(line: &str) -> String{
    let STRING_REGEX = Regex::new( r#"^"(([^"\\]|\\.)*)"$"#).unwrap(); // Soporte para cadenas
    let CHAR_REGEX = Regex::new( r"^'([^'\\]|\\.)'$").unwrap(); // Soporte para un solo caracter

    let res: String = STRING_REGEX.replace_all(line, "STR").to_string();
    let res: String = CHAR_REGEX.replace_all(&res, "CHR").to_string();

    res
}

fn replace_tokens(token: &str, line: u16) -> Result<&str, CompilerError> {
    let regexMap = indexmap! (
    "|STR|" => Regex::new("^STR$"),
    "|CHR|" => Regex::new("^CHR$"),
    "|WH|" => Regex::new(r"^(while)$"),
    "|IF|" => Regex::new(r"^(if)$"),
    "|CASE|" => Regex::new(r"^(case)$"),
    "|DO|" => Regex::new(r"^(do)$"),
    "|DT|" => Regex::new(r"^(int|char|float|double|void)$"),
    "|RW|" => Regex::new( r"^(auto|else|long|switch|break|enum|register|typedef|case|extern|return|union|char|float|short|unsigned|const|for|signed|void|continue|goto|sizeof|volatile|default|if|static|while|do|int|struct|_Packed|double)$"),
    "|ID|" => Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]{0,30}$"),
    "|NU|" => Regex::new(r"^[0-9]+$"),
    "|CA|" => Regex::new(r"^(\+=|-=|\*=|/=|\+\+|--)$"),
    "|OP|" => Regex::new(r"^[+\-*/%\^!]$"),
    "|LO|" => Regex::new(r"^(&&|\|\|)$"),
    "|BO|" => Regex::new(r"^(&|\||\^|~|<<|>>)$"),
    "|CO|" => Regex::new(r"^(==|!=|<|<=|>|>=|<>)$"),
    "|AS|" => Regex::new(r"^=$"),
    "|LP|" => Regex::new(r"^\($"),
    "|RP|" => Regex::new(r"^\)$"),
    "|LB|" => Regex::new(r"^\{$"),
    "|RB|" => Regex::new(r"^\}$"),
    "|LBR|" => Regex::new(r"^\[$"),
    "|RBR|" => Regex::new(r"^\]$"),
    "|COMA|" => Regex::new(r"^,$"),
    "|DOT|" => Regex::new(r"^\.$"),
    "|SC|" => Regex::new(r"^;$"),
    "|COL|" => Regex::new(r"^:$"),
    );

    let mut family: &str = "";

    for (regexFam, regex) in regexMap {
        match regex {
            Ok(re) => {
                if re.is_match(token) {
                    family = regexFam;
                    break;
                }
            }
            Err(e) => return Err(CompilerError::RegexError(e)),
        }
    }

    if family.is_empty() {
        return Err(CompilerError::InvalidToken(token.to_owned(), line));
    }

    Ok(family)
}

fn check_syntax(line: &str, lineN: u16) -> Result<String, CompilerError> {
    let mut regexMap = BTreeMap::new();

    regexMap.insert("INITIALIZATION_LINE", Regex::new(
        r"^\|DT\|\|ID\|((\|AS\|(\|ID\||\|NU\|)){0,1}(\|OP\|(\|ID\||\|NU\|))*)(\|COMA\|\|ID\|((\|AS\|(\|ID\||\|NU\|)){0,1}(\|OP\|(\|ID\||\|NU\|))*))*\|SC\|$"
    ));
    regexMap.insert(
        "ASSIGNATION_LINE",
        Regex::new(r"^\|ID\|((\|AS\|(\|ID\||\|NU\|)){1}(\|OP\|(\|ID\||\|NU\|))*)\|SC\|$"),
    );
    regexMap.insert(
        "FUNCTION_LINE",
        Regex::new(r"^\|DT\|\|ID\|\|LP\|(\|DT\|\|ID\|){0,1}(\|COMA\|\|DT\|\|ID\|)*\|RP\|\|LB\|$"),
    );
    regexMap.insert("IF_LINE", Regex::new(
    r"^\|IF\|\|LP\|((\|ID\||\|NU\|)\|CO\|(\|ID\||\|NU\|)){1}(\|LO\|(\|ID\||\|NU\|)\|CO\|(\|ID\||\|NU\|))*\|RP\|\|LB\|$"),
    );
    regexMap.insert("WHILE_LINE", Regex::new(
r"^\|WH\|\|LP\|((\|ID\||\|NU\|)\|CO\|(\|ID\||\|NU\|)){1}(\|LO\|(\|ID\||\|NU\|)\|CO\|(\|ID\||\|NU\|))*\|RP\|\|LB\|$"),);
    regexMap.insert("END_BLOCK_LINE", Regex::new(r"^\|RB\|$"));
    regexMap.insert("DO_LINE", Regex::new(r"^\|DO\|\|LB\|$"));
    regexMap.insert("DO_BLOCK_END", Regex::new(
r"^\|WH\|\|LP\|((\|ID\||\|NU\|)\|CO\|(\|ID\||\|NU\|)){1}(\|LO\|(\|ID\||\|NU\|)\|CO\|(\|ID\||\|NU\|))*\|RP\|\|SC\|$"),);
    regexMap.insert("SEMICOLON_LINE", Regex::new(r"^\|SC\|$"));

    for (regexLine, regex) in regexMap {
        match regex {
            Ok(re) => {
                if re.is_match(line) {
                    return Ok(format!("Linea {} Valida: {}", lineN, regexLine));
                }
            }
            Err(e) => return Err(CompilerError::RegexError(e)),
        }
    }

    Err(CompilerError::InvalidSyntax(lineN))
}
