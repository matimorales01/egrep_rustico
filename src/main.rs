use std::env;

use trabajo_practico::{grep_error::GrepError, grep_rustico::GrepRustico};

///Se ejecuta el codigo para leer la linea de comandos de la terminal, y si no hay errores se corre el programa
fn egrep(args: Vec<String>) -> Result<(), GrepError> {
    let mut grep = match GrepRustico::leer_comandos(args) {
        Ok(grep) => grep,
        Err(e) => return Err(e),
    };

    let _ = grep.run();

    Ok(())
}

///Se toman los valores de la linea de comandos y se ejecuta el programa.
fn main() {
    let args: Vec<String> = env::args().collect();

    let _ = egrep(args);
}
