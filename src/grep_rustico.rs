use std::{
    fs::File,
    io::{BufRead, BufReader, Lines},
};

use crate::{grep_error::GrepError, regex::Regex};

/// Representa un grep simple implementado en Rust.
#[derive(Debug)]
pub struct GrepRustico {
    file: File,
    regex_vec: Vec<Regex>,
}

impl GrepRustico {
    /// Lee los argumentos pasados por línea de comandos para inicializar un `GrepRustico`.
    ///
    /// # Arguments
    ///
    /// * `args` - Un vector de cadenas que representa los argumentos de la línea de comandos.
    ///
    /// # Returns
    ///
    /// Devuelve un `GrepRustico` inicializado si los argumentos son válidos y no hay errores.
    ///
    /// Si hay un error en los argumentos o al abrir el archivo, devuelve un error de tipo `GrepError`.
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

    /// Ejecuta el grep en el archivo y devuelve un vector de las líneas que coinciden con las expresiones regulares.
    ///
    /// # Returns
    ///
    /// Devuelve un vector de cadenas que representan las líneas que coinciden con las expresiones regulares.
    ///
    /// Si hay un error al leer el archivo o al ejecutar el grep, devuelve un error de tipo `GrepError`.
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
    /// Imprime las líneas que coinciden con las expresiones regulares.
    ///
    /// # Arguments
    ///
    /// * `matches` - Un vector de cadenas que representan las líneas que coinciden con las expresiones regulares.
    fn imprimir_matches(&self, matches: &Vec<String>) {
        for linea in matches {
            println!("{}", linea);
        }
    }

    /// Abre un archivo dado su nombre.
    ///
    /// # Arguments
    ///
    /// * `nombre_archivo` - El nombre del archivo que se va a abrir.
    ///
    /// # Returns
    ///
    /// Devuelve un objeto `File` si el archivo se abre con éxito.
    ///
    /// Si hay un error al abrir el archivo, devuelve un error de tipo `GrepError`.
    fn abrir_archivo(nombre_archivo: &str) -> Result<File, GrepError> {
        match File::open(nombre_archivo) {
            Ok(file) => Ok(file),
            Err(_) => Err(GrepError::ErrArchivo),
        }
    }

    /// Lee todas las palabras del archivo y las devuelve como un vector de cadenas.
    ///
    /// # Returns
    ///
    /// Devuelve un vector de cadenas que representan todas las palabras del archivo.
    ///
    /// Si hay un error al leer el archivo, devuelve un error de tipo `GrepError`.
    fn leer_palabras(&self) -> Result<Vec<String>, GrepError> {
        let lector_lineas: Lines<BufReader<&File>> = BufReader::new(&self.file).lines();

        let cadenas = GrepRustico::leer_archivo(lector_lineas)?;

        Ok(cadenas)
    }

    /// Lee un archivo línea por línea y lo convierte en un vector de cadenas.
    ///
    /// # Arguments
    ///
    /// * `lector_lineas` - Un iterador sobre las líneas del archivo.
    ///
    /// # Returns
    ///
    /// Devuelve un vector de cadenas que representan las líneas del archivo.
    ///
    /// Si hay un error al leer el archivo, devuelve un error de tipo `GrepError`.
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

    /// Filtra cada línea y ejecuta el grep para cada expresión regular.
    ///
    /// # Arguments
    ///
    /// * `lines` - Un vector de cadenas que representan las líneas del archivo.
    ///
    /// # Returns
    ///
    /// Devuelve un vector de cadenas que representan las líneas que coinciden con las expresiones regulares.
    ///
    /// Si hay un error al ejecutar el grep, devuelve un error de tipo `GrepError`.
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
