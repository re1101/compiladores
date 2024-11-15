use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use regex::Regex;

fn main() -> io::Result<()> {
    
    let path = "prueba.txt"; // Ubicacion del archivo de texto
    let file = File::open(path)?;

    
    let regex_caracteres = Regex::new(
        r"^[ a-zA-Z0-9_+*\(\)\[\]#&/|=<>%\:!]+;$" // Expresión regular para validar los caracteres en cada línea
    ).unwrap();

    let regex_reservadas = Regex::new(
        r"\b(auto|else|long|switch|break|enum|register|typedef|case|extern|return|union|char|float|short|unsigned|const|for|signed|void|continue|goto|sizeof|volatile|default|if|static|while|do|int|struct|_Packed|double)\b" // Expresión regular para identificar palabras reservadas
    ).unwrap();

    let regex_identificadores = Regex::new(
        r"^[a-zA-Z_][a-zA-Z0-9_]{0,30}$" // Expresión regular para validar identificadores
    ).unwrap();

    // Leer el archivo línea por línea
    if let Ok(lines) = read_lines(path) {
        let mut buffer = String::new();
        for line in lines {
            if let Ok(line) = line {
                buffer.push_str(&line);  // Agrega la línea al buffer

                // Procesar cuando encontramos un ';' al final de la línea acumulada
                if buffer.contains(';') {
                    // Clonar el buffer antes de procesar
                    let buffer_cloned = buffer.clone();
                    
                    // Separa por el ';' para procesar cada "línea lógica" de código
                    let partes: Vec<&str> = buffer_cloned.split(';').collect();
                    
                    // Extraemos la última parte que puede estar incompleta y la volvemos a poner en el buffer
                    buffer = partes.last().unwrap_or(&"").to_string();
                    
                    // Procesamos las partes completas de código
                    for parte in partes.iter().take(partes.len() - 1) {
                        let linea_completa = parte.trim().to_owned() + ";";
                        println!("Procesando línea: {}", linea_completa);

                        // Verificar si los caracteres de la línea pertenecen al lenguaje C
                        if regex_caracteres.is_match(&linea_completa) {
                            println!("Los caracteres pertenecen al lenguaje C.");
                        } else {
                            println!("Los caracteres NO pertenecen al lenguaje C.");
                        }

                        // Separar la línea por espacios y analizar cada palabra
                        let tokens: Vec<&str> = linea_completa.split_whitespace().collect();
                        for token in tokens {
                            if regex_reservadas.is_match(token) {
                                println!("Palabra reservada encontrada: {}", token);
                            } else if regex_identificadores.is_match(token) {
                                println!("Identificador encontrado: {}", token);
                            } else {
                                println!("Token desconocido: {}", token);
                            }
                        }
                    }
                }
            }
        }

        // Procesar cualquier línea restante en el buffer (si falta un ';' al final)
        if !buffer.is_empty() {
            buffer.push(';'); // Añadimos el ';' faltante si es necesario
            println!("Procesando línea: {}", buffer);

            if regex_caracteres.is_match(&buffer) {
                println!("Los caracteres pertenecen al lenguaje C.");
            } else {
                println!("Los caracteres NO pertenecen al lenguaje C.");
            }

            let tokens: Vec<&str> = buffer.split_whitespace().collect();
            for token in tokens {
                if regex_reservadas.is_match(token) {
                    println!("Palabra reservada encontrada: {}", token);
                } else if regex_identificadores.is_match(token) {
                    println!("Identificador encontrado: {}", token);
                } else {
                    println!("Token desconocido: {}", token);
                }
            }
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
