use regex::Regex;
use std::{
    error::Error,
    fmt,
    fs::File,
    io::{self, BufRead},
    path::Path,
    collections::BTreeMap,
};
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
            CompilerError::RegexError(err) => Some(err),
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
            //println!("DEBUG: Line => {:?}", line);

            if let Ok(line) = line {
                linesBuffer.push_str(&line); // Agrega la línea al buffer

                //println!("DEBUG: linesBuffer => {:?}", linesBuffer);
            }
        }

        // Expresión regular para capturar palabras con sus delimitadores
        let re = Regex::new(r"[^\s{};]+(?:\s[^\s{};]+)*[{};]?|[{};]").unwrap();
        // Buscar coincidencias en la cadena
        let logicalLines: Vec<&str> = re.find_iter(&linesBuffer).map(|m| m.as_str()).collect();

        //println!("DEBUG: logicalLines => {:?}", logicalLines);

        let mut currentLine: u16 = 0;

        for logicalLine in logicalLines {
            //println!("DEBUG: logicalLine => {:?}", logicalLine);

            let logicalLine = &logicalLine.replace("&&", "TEMPAND");
            let logicalLine = &logicalLine.replace("||", "TEMPOR");
            let logicalLine = &logicalLine.replace("==", "TEMPEQUALS");
            let logicalLine = &logicalLine.replace(">=", "TEMPEQG");
            let logicalLine = &logicalLine.replace("<=", "TEMPEQL");
            let logicalLine = &logicalLine.replace("!=", "TEMPNOT");
            let logicalLine = &logicalLine.replace("<>", "TEMPDIFF");

            //println!("DEBUG: first replace logicalLine => {:?}", logicalLine);

            // Expresión regular para capturar palabras, simbolos y delimitadores
            let reT = Regex::new(r"\w+|[^\w\s]").unwrap();
            // Buscar coincidencias en la cadena
            let tempTokens: Vec<&str> = reT.find_iter(&logicalLine).map(|m| m.as_str()).collect();

            //println!("DEBUG: tempTokens => {:?}", tempTokens);

            let mut i: usize = 0;
            let mut tokens: Vec<&str> = Vec::new();

            for tempToken in tempTokens {
                //println!("DEBUG: tempToken => {:?}", tempToken);

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

                i += 1;
            }

            //println!("DEBUG: tokens => {:?}", tokens);

            let mut families: Vec<&str> = Vec::new();

            for token in tokens {
                //Temporary Assignment

                //println!("DEBUG: token => {:?}", token);

                families.push(match replace_tokens(token, currentLine) {
                    Ok(val) => val,
                    Err(e) => return Err( e ) ,
                });
            }

            let mut newLine: String = families.into_iter().collect();

            println!("DEBUG: Families Line => {:?}", newLine);

            println!("{:#?}", check_syntax(&newLine, currentLine));

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

fn replace_tokens(token: &str, line: u16) -> Result<&str, CompilerError> {
    let mut regexMap = BTreeMap::new();

    regexMap.insert( "|WH|" , Regex::new( r"while"));
    regexMap.insert( "|IF|" , Regex::new( r"if"));
    regexMap.insert( "|CASE|" , Regex::new( r"case"));
    regexMap.insert( "|DO|" , Regex::new( r"do"));
    regexMap.insert( "|DT|" , Regex::new( r"(int|char|float|double|void)"));
    regexMap.insert( "|RW|" , Regex::new( r"(auto|else|long|switch|break|enum|register|typedef|case|extern|return|union|char|float|short|unsigned|const|for|signed|void|continue|goto|sizeof|volatile|default|if|static|while|do|int|struct|_Packed|double)"));
    regexMap.insert( "|ID|" , Regex::new( r"[a-zA-Z_][a-zA-Z0-9_]{0,30}"));
    regexMap.insert( "|NUM|" , Regex::new( r"[0-9]+"));
    regexMap.insert( "|CA|" , Regex::new( r"(\+=|-=|\*=|/=|\+\+|--)"));
    regexMap.insert( "|CA|" , Regex::new( r"(\+=|-=|\*=|/=|\+\+|--)"));
    regexMap.insert( "|LO|" , Regex::new( r"(&&|\|\|)"));
    regexMap.insert( "|BO|" , Regex::new( r"(&|\||\^|~|<<|>>)"));
    regexMap.insert( "|COM|" , Regex::new( r"(==|!=|<|<=|>|>=|<>)"));
    regexMap.insert( "|AS|" , Regex::new( r"="));
    regexMap.insert( "|LP|" , Regex::new( r"\("));
    regexMap.insert( "|RP|" , Regex::new( r"\)"));
    regexMap.insert( "|LB|" , Regex::new( r"\{"));
    regexMap.insert( "|RB|" , Regex::new( r"\}"));
    regexMap.insert( "|LBR|" , Regex::new( r"\["));
    regexMap.insert( "|RBR|" , Regex::new( r"\]"));
    regexMap.insert( "|COMA|" , Regex::new( r","));
    regexMap.insert( "|DOT|" , Regex::new( r"\."));
    regexMap.insert( "|SC|" ,Regex::new( r";"));
    regexMap.insert( "|COL|" , Regex::new( r":"));
    
    //regexMap.insert( "|STR|" , Regex::new( r#"(([^"\\]|\\.)*)"#)); // Soporte para cadenas
    //regexMap.insert( "|CHR|" , Regex::new( r"'([^'\\]|\\.){1}'")); // Soporte para un solo caracter
    
    //regexMap.insert( SINGLE_LINE_COMMENT_REGEX , Regex::new( r"\\/\\/.*")); // Soporte para comentarios unilinea
    //regexMap.insert( MULTI_LINE_COMMENT_REGEX , Regex::new( r"/\\*([^*]|\\*(?!/))*\\*/")); // Soporte para comentarios multilinea
    
    let mut family: &str = "";

    println!("DEBUG: token => {}", token);

    for (regexFam, regex) in regexMap {
        match regex {
            Ok(re) => if re.is_match(token) {
                family = regexFam;
                println!("DEBUG: family => {}", family);
                break;
            },
            Err(e) => return Err(CompilerError::RegexError(e)),
        }
    }

    if family.is_empty(){
        return Err( CompilerError::InvalidToken( token.to_owned(), line ) )
    }

    Ok(family)
}

fn check_syntax(line: &str, lineN: u16) -> Result<String, CompilerError> {
    let mut regexMap = BTreeMap::new();

    regexMap.insert("RESERVED_WORDS", Regex::new(
        r"\b(auto|else|long|switch|break|enum|register|typedef|case|extern|return|union|char|float|short|unsigned|const|for|signed|void|continue|goto|sizeof|volatile|default|if|static|while|do|int|struct|_Packed|double)\b"
    ).unwrap());
    regexMap.insert(
        "IDENTIFIER",
        Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]{0,30}$").unwrap(),
    );
    regexMap.insert("NUMBER", Regex::new(r"[0-9]+").unwrap());

    for (regexLine, regex) in regexMap {
        if regex.is_match(line) {
            return Ok(format!("Linea {} Valida: {}", lineN, regexLine));
        }
    }

    Err( CompilerError::InvalidSyntax( lineN ) )
}
