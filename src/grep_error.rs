use std::fmt;

#[derive(Debug)]
///Maneja los errores del programa
pub enum GrepError {
    Err,
    ErrArchivo,
}

impl fmt::Display for GrepError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GrepError::Err => write!(f, ""),
            GrepError::ErrArchivo => write!(f, "No existe el archivo o el directorio"),
        }
    }
}
