use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

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
            println!("DEBUG: Line => {:?}", line);

            if let Ok(line) = line {
                linesBuffer.push_str(&line); // Agrega la línea al buffer

                println!("DEBUG: linesBuffer => {:?}", linesBuffer);
            }
        }

        // Expresión regular para capturar palabras con sus delimitadores
        let re = Regex::new(r"[^{\s};]+(?:\s[^{\s};]+)*[{};]?").unwrap();
        // Buscar coincidencias en la cadena
        let partes: Vec<&str> = re.find_iter(&linesBuffer).map(|m| m.as_str()).collect();

        println!("{:?}", partes);
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
