use std::{
    fs::File,
    io::{BufRead, BufReader, Lines},
};

use crate::{grep_error::GrepError, regex::Regex};

#[derive(Debug)]
pub struct GrepRustico {
    file: File,
    regex_vec: Vec<Regex>,
}

impl GrepRustico {
    ///Se reciben los argumentos pasados por linea de comandos: cargo run -regularexpression- nombrearchivo
    /// se chequea que esten todos los necesarios, y ya con los argumentos obtenidos se llama a abrir archivo
    /// ante cualquier error se retorna error, de lo contrario se devuelve la estructura GrepRustico
    pub fn leer_comandos(args: Vec<String>) -> Result<GrepRustico, GrepError> {
        if args.len() != 3 {
            return Err(GrepError::Err);
        }

        let regex = &args[1];
        let nombre_archivo = &args[2];

        let file = GrepRustico::abrir_archivo(nombre_archivo)?;
        let regex_vec = Regex::crear_regex(regex)?;

        Ok(GrepRustico { file, regex_vec })
    }
    ///Se corren todos los procedimientos para la lectura del archivo, el cual se tiene en formato de vector, y se va
    /// leyendo palabra por palabra, para ir matcheando, y ante una palabra matcheada, se printea.
    /// ante cualquier error se retorna GrepError
    pub fn run(&mut self) -> Result<Vec<String>, GrepError> {
        let mut matches = Vec::new();
        let cadena: Vec<String> = match self.leer_palabras() {
            Ok(cadena) => cadena,
            Err(_err) => return Err(GrepError::ErrArchivo),
        };

        match self.filtrar_cadena_y_grep(&cadena) {
            Ok(results) => {
                for result in results {
                    matches.push(result);
                }
            }
            Err(_err) => return Err(GrepError::ErrArchivo),
        };

        self.imprimir_matches(&matches);
        Ok(matches)
    }

    ///Funcion auxiliar para imprimir los strings que seran las palabras matcheadas
    fn imprimir_matches(&self, matches: &Vec<String>) {
        for linea in matches {
            println!("{}", linea);
        }
    }
    ///Funcion auxiliar para realizar la apertura del archivo, ante un error devuelve GrepError    
    fn abrir_archivo(nombre_archivo: &str) -> Result<File, GrepError> {
        match File::open(nombre_archivo) {
            Ok(file) => Ok(file),
            Err(_) => Err(GrepError::ErrArchivo),
        }
    }
    ///Funcion auxiliar para leer cada linea del archivo de manera eficiente. Devuelve un vector de cadenas de string
    /// o en caso de error un GrepError
    fn leer_palabras(&self) -> Result<Vec<String>, GrepError> {
        let lector_lineas: Lines<BufReader<&File>> = BufReader::new(&self.file).lines();

        let cadenas = GrepRustico::leer_archivo(lector_lineas)?;

        Ok(cadenas)
    }

    ///Funcion auxiliar para armar un vector de cadenas de string. Devuelve GrepError en caso de error
    fn leer_archivo(lector_lineas: Lines<BufReader<&File>>) -> Result<Vec<String>, GrepError> {
        let mut cadenas: Vec<String> = Vec::new();

        for linea in lector_lineas {
            match linea {
                Ok(linea) => cadenas.push(linea),
                Err(_) => return Err(GrepError::ErrArchivo),
            };
        }

        Ok(cadenas)
    }
    ///Se filtra palabra por palabra, se chequea que sea ascii, y se corre el procedimiento del grep en cada regex.
    /// En caso de error devuelve GrepError. Caso contrario un vector de resultados
    fn filtrar_cadena_y_grep(&mut self, lines: &Vec<String>) -> Result<Vec<String>, GrepError> {
        let mut resultado = Vec::new();

        for line in lines {
            if !line.is_ascii() {
                return Err(GrepError::Err);
            }

            for regex in &mut self.regex_vec {
                if !resultado.contains(line) && regex.test(line)? {
                    resultado.push(line.clone());
                }
            }
        }
        Ok(resultado)
    }
}
