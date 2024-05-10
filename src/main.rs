use std::env;
use trabajo_practico::{grep_error::GrepError, grep_rustico::GrepRustico};

/// Ejecuta el código para leer la línea de comandos de la terminal y, si no hay errores, corre el programa.
///
/// # Arguments
///
/// * `args` - Un vector de cadenas que representa los argumentos de la línea de comandos.
///
/// # Returns
///
/// Devuelve `Ok(())` si el programa se ejecuta correctamente, de lo contrario devuelve un error de tipo `GrepError`.
fn egrep(args: Vec<String>) -> Result<(), GrepError> {
    let mut grep = match GrepRustico::leer_comandos(args) {
        Ok(grep) => grep,
        Err(e) => return Err(e),
    };

    let _ = grep.run();

    Ok(())
}

/// Toma los valores de la línea de comandos y ejecuta el programa.
fn main() {
    let args: Vec<String> = env::args().collect();

    let _ = egrep(args);
}
