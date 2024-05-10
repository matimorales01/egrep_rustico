use crate::{grep_error::GrepError, regex_value::RegexValue};
use std::str::Chars;

/// Permite manejar las expresiones regulares de las diferentes clases de caracteres y bracket expressions
#[derive(Debug, PartialEq, Clone)]
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

impl RegexClase {
    /// Verifica si el caracter coincide con la clase de caracteres especificada y devuelve un booleano.
    ///
    /// # Arguments
    ///
    /// * `caracter` - El caracter que se va a validar.
    ///
    /// # Returns
    ///
    /// Devuelve `true` si el caracter coincide con la clase de caracteres especificada,
    /// de lo contrario, devuelve `false`.
    pub fn validar_caracter(&self, caracter: char) -> bool {
        match self {
            RegexClase::AlNum => caracter.is_ascii_alphanumeric(),
            RegexClase::Alpha => caracter.is_ascii_alphabetic(),
            RegexClase::Digit => caracter.is_ascii_digit(),
            RegexClase::Lower => caracter.is_ascii_lowercase(),
            RegexClase::Upper => caracter.is_ascii_uppercase(),
            RegexClase::Space => caracter.is_ascii_whitespace(),
            RegexClase::Punct => caracter.is_ascii_punctuation(),
            RegexClase::Custom(chars, negado) => {
                if *negado {
                    !chars.contains(&caracter)
                } else {
                    chars.contains(&caracter)
                }
            }
        }
    }

    /// Lee y procesa una clase de caracteres y devuelve su representación como `RegexValue`.
    ///
    /// # Arguments
    ///
    /// * `chars_iter` - Un iterador de caracteres que representa la clase de caracteres.
    ///
    /// # Returns
    ///
    /// Devuelve `Ok(RegexValue)` si la clase de caracteres se procesa correctamente
    /// y se devuelve su representación como `RegexValue`.
    ///
    /// Devuelve `Err(GrepError)` si ocurre algún error durante el procesamiento de la clase de caracteres.
    pub fn read_character_class(chars_iter: &mut Chars) -> Result<RegexValue, GrepError> {
        let mut class_content = String::new();
        for char_in_class in chars_iter.by_ref() {
            if char_in_class == ']' {
                class_content.push(char_in_class);
                break;
            }
            class_content.push(char_in_class);
        }


        if chars_iter.next() != Some(']') {
            return Err(GrepError::Err);
        }

        let regex_clase = match class_content.as_str() {
            "[:alnum:]" => RegexClase::AlNum,
            "[:alpha:]" => RegexClase::Alpha,
            "[:digit:]" => RegexClase::Digit,
            "[:lower:]" => RegexClase::Lower,
            "[:upper:]" => RegexClase::Upper,
            "[:space:]" => RegexClase::Space,
            "[:punct:]" => RegexClase::Punct,
            _ => {
                return Err(GrepError::Err);
            }
        };

        Ok(RegexValue::Clase(regex_clase))
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validar_caracter_alnum() {
        let clase = RegexClase::AlNum;
        assert_eq!(clase.validar_caracter('a'), true);
        assert_eq!(clase.validar_caracter('1'), true);
        assert_eq!(clase.validar_caracter('?'), false);
    }

    #[test]
    fn test_validar_caracter_alpha() {
        let clase = RegexClase::Alpha;
        assert_eq!(clase.validar_caracter('a'), true);
        assert_eq!(clase.validar_caracter('A'), true);

        assert_eq!(clase.validar_caracter('1'), false);
        assert_eq!(clase.validar_caracter('$'), false);
    }

    #[test]
    fn test_validar_caracter_digit() {
        let clase = RegexClase::Digit;
        assert_eq!(clase.validar_caracter('a'), false);
        assert_eq!(clase.validar_caracter('1'), true);
        assert_eq!(clase.validar_caracter('*'), false);
    }

    #[test]
    fn test_validar_caracter_lower() {
        let clase = RegexClase::Lower;
        assert_eq!(clase.validar_caracter('a'), true);
        assert_eq!(clase.validar_caracter('A'), false);
        assert_eq!(clase.validar_caracter('*'), false);
    }

    #[test]
    fn test_validar_caracter_upper() {
        let clase = RegexClase::Upper;
        assert_eq!(clase.validar_caracter('a'), false);
        assert_eq!(clase.validar_caracter('A'), true);
        assert_eq!(clase.validar_caracter('*'), false);
    }

    #[test]
    fn test_validar_caracter_space() {
        let clase = RegexClase::Space;
        assert_eq!(clase.validar_caracter(' '), true);
        assert_eq!(clase.validar_caracter('\t'), true);
        assert_eq!(clase.validar_caracter('a'), false);
    }

    #[test]
    fn test_validar_caracter_punct() {
        let clase = RegexClase::Punct;
        assert_eq!(clase.validar_caracter('.'), true);
        assert_eq!(clase.validar_caracter('a'), false);
    }

    #[test]
    fn test_validar_caracter_custom() {
        let clase = RegexClase::Custom(vec!['m', 'a', 't', 'i'], false);
        assert_eq!(clase.validar_caracter('m'), true);
        assert_eq!(clase.validar_caracter('z'), false);

        let clase_negada = RegexClase::Custom(vec!['m', 'a', 't'], true);
        assert_eq!(clase_negada.validar_caracter('t'), false);
    }
}
