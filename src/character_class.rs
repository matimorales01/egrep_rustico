use crate::{grep_error::GrepError, regex_value::RegexValue};
use std::str::Chars;

/// Permite manejar las expresiones regulares de las diferentes clases de caracteres y bracket expressions
#[derive(Debug, PartialEq, Clone)]
pub enum CharacterClass {
    AlNum,
    Alpha,
    Digit,
    Lower,
    Upper,
    Space,
    Punct,
    Custom(Vec<char>, bool), // Agregar la variante Custom
}

impl CharacterClass {
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
    pub fn valid_character(&self, caracter: char) -> bool {
        match self {
            CharacterClass::AlNum => caracter.is_ascii_alphanumeric(),
            CharacterClass::Alpha => caracter.is_ascii_alphabetic(),
            CharacterClass::Digit => caracter.is_ascii_digit(),
            CharacterClass::Lower => caracter.is_ascii_lowercase(),
            CharacterClass::Upper => caracter.is_ascii_uppercase(),
            CharacterClass::Space => caracter.is_ascii_whitespace(),
            CharacterClass::Punct => caracter.is_ascii_punctuation(),
            CharacterClass::Custom(chars, negado) => {
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
            "[:alnum:]" => CharacterClass::AlNum,
            "[:alpha:]" => CharacterClass::Alpha,
            "[:digit:]" => CharacterClass::Digit,
            "[:lower:]" => CharacterClass::Lower,
            "[:upper:]" => CharacterClass::Upper,
            "[:space:]" => CharacterClass::Space,
            "[:punct:]" => CharacterClass::Punct,
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
        let clase = CharacterClass::AlNum;
        assert_eq!(clase.valid_character('a'), true);
        assert_eq!(clase.valid_character('1'), true);
        assert_eq!(clase.valid_character('?'), false);
    }

    #[test]
    fn test_validar_caracter_alpha() {
        let clase = CharacterClass::Alpha;
        assert_eq!(clase.valid_character('a'), true);
        assert_eq!(clase.valid_character('A'), true);

        assert_eq!(clase.valid_character('1'), false);
        assert_eq!(clase.valid_character('$'), false);
    }

    #[test]
    fn test_validar_caracter_digit() {
        let clase = CharacterClass::Digit;
        assert_eq!(clase.valid_character('a'), false);
        assert_eq!(clase.valid_character('1'), true);
        assert_eq!(clase.valid_character('*'), false);
    }

    #[test]
    fn test_validar_caracter_lower() {
        let clase = CharacterClass::Lower;
        assert_eq!(clase.valid_character('a'), true);
        assert_eq!(clase.valid_character('A'), false);
        assert_eq!(clase.valid_character('*'), false);
    }

    #[test]
    fn test_validar_caracter_upper() {
        let clase = CharacterClass::Upper;
        assert_eq!(clase.valid_character('a'), false);
        assert_eq!(clase.valid_character('A'), true);
        assert_eq!(clase.valid_character('*'), false);
    }

    #[test]
    fn test_validar_caracter_space() {
        let clase = CharacterClass::Space;
        assert_eq!(clase.valid_character(' '), true);
        assert_eq!(clase.valid_character('\t'), true);
        assert_eq!(clase.valid_character('a'), false);
    }

    #[test]
    fn test_validar_caracter_punct() {
        let clase = CharacterClass::Punct;
        assert_eq!(clase.valid_character('.'), true);
        assert_eq!(clase.valid_character('a'), false);
    }

    #[test]
    fn test_validar_caracter_custom() {
        let clase = CharacterClass::Custom(vec!['m', 'a', 't', 'i'], false);
        assert_eq!(clase.valid_character('m'), true);
        assert_eq!(clase.valid_character('z'), false);

        let clase_negada = CharacterClass::Custom(vec!['m', 'a', 't'], true);
        assert_eq!(clase_negada.valid_character('t'), false);
    }
}
