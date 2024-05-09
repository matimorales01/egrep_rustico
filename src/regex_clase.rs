#[derive(Debug, PartialEq, Clone)]

///Permite manejar las expresiones regulares de las diferentes clases de caracteres.
pub enum RegexClase {
    AlNum,
    Alpha,
    Digit,
    Lower,
    Upper,
    Space,
    Punct,
    Custom(Vec<char>, bool), // Agregar la variante Custom
}
///Verifica si el caracter coincide con la clase de caracteres especificada, devuelve un booleano.
impl RegexClase {
    pub fn validar_caracter(&self, caracter: char) -> bool {
        match self {
            RegexClase::AlNum => {
                if caracter.is_ascii_alphanumeric() {
                    return true;
                }
                false
            }
            RegexClase::Alpha => {
                if caracter.is_ascii_alphabetic() {
                    return true;
                }
                false
            }
            RegexClase::Digit => {
                if caracter.is_ascii_digit() {
                    return true;
                }
                false
            }
            RegexClase::Lower => {
                if caracter.is_ascii_lowercase() {
                    return true;
                }
                false
            }
            RegexClase::Upper => {
                if caracter.is_ascii_uppercase() {
                    return true;
                }
                false
            }
            RegexClase::Space => {
                if caracter.is_ascii_whitespace() {
                    return true;
                }
                false
            }
            RegexClase::Punct => {
                if caracter.is_ascii_punctuation() {
                    return true;
                }
                false
            }
            RegexClase::Custom(chars, negado) => {
                if *negado {
                    !chars.contains(&caracter)
                } else {
                    chars.contains(&caracter)
                }
            }
        }
    }
}
