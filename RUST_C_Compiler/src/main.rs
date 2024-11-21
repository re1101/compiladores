use regex::Regex;
use std::{fs::File, fmt, error::Error, io::{self, BufRead},path::Path};

#[derive(Debug)]
enum CompilerError {
    OpenError(std::io::Error),
    InvalidSyntax(u16),
    InvalidToken(String, u16)
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompilerError::OpenError(err) => write!(f, "Captured Underlying Error: {}", err),
            CompilerError::InvalidSyntax(line) => write!(f, "Invalid syntax at line: {}", line),
            CompilerError::InvalidToken(token,line) => write!(f, "Invalid token \"{}\" at line: {}", token, line)
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

fn main() -> io::Result<()> {
    let path = "../prueba.txt"; // Ubicacion del archivo de texto
    let _file = File::open(path)?;

    let regex_caracteres = Regex::new(
        r"^[ a-zA-Z0-9_+*\(\)\[\]#&/|=<>%\:!]+;$", // Expresión regular para validar los caracteres en cada línea
    )
    .unwrap();

    let regex_reservadas = Regex::new(
        r"\b(auto|else|long|switch|break|enum|register|typedef|case|extern|return|union|char|float|short|unsigned|const|for|signed|void|continue|goto|sizeof|volatile|default|if|static|while|do|int|struct|_Packed|double)\b", // Expresión regular para identificar palabras reservadas
    )
    .unwrap();

    let regex_identificadores = Regex::new(
        r"^[a-zA-Z_][a-zA-Z0-9_]{0,30}$", // Expresión regular para validar identificadores
    )
    .unwrap();

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
            let tempTokens: Vec<&str> =
                reT.find_iter(&logicalLine).map(|m| m.as_str()).collect();

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

            let mut newLine: Vec<&str> = Vec::new();

            for token in tokens {
                let mut families: Vec<&str> = Vec::new(); //Temporary Assignment

                //TODO replace token with it's family

                println!("DEBUG: token => {:?}", token);

                families.push( match replace_tokens(token, currentLine){
                    Ok(val) => val,
                    Err(e) => panic!("|ERROR|: ", e)
                }
            }
            }

            currentLine += 1;
        }
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

fn replace_tokens( token:&str, line: u16 ) -> Result< &str, CompilerError >{
    let mut family: &str;

    family = match token {
       "" => "",
       _ => return Err(CompilerError::InvalidToken(token.to_string(), line))
    };

    Ok(family)
}

fn check_syntax() -> Result<Vec<&str>, CompilerError>{
    Ok(families)
    Err()
}
