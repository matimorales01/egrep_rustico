use crate::regex_clase::RegexClase;

/// Representa un valor en una expresión regular, que puede ser un carácter literal, un comodín o una clase de caracteres.
#[derive(Debug, Clone, PartialEq)]
pub enum RegexValue {
    Literal(char),
    Wildcard,
    Clase(RegexClase),
}

impl RegexValue {
    /// Devuelve la longitud de la coincidencia del valor en el texto.
    ///
    /// # Arguments
    ///
    /// * `text` - El texto en el que se va a buscar la coincidencia.
    ///
    /// # Returns
    ///
    /// Devuelve la longitud de la coincidencia del valor en el texto.
    ///
    /// Si no se encuentra ninguna coincidencia, devuelve 0.
    pub fn matches(&self, text: &str) -> usize {
        match self {
            RegexValue::Literal(c) => {
                for (i, c_text) in text.chars().enumerate() {
                    if c_text == *c {
                        return i + c.len_utf8();
                    }
                }
                0
            }
            RegexValue::Wildcard => {
                if let Some(c) = text.chars().next() {
                    c.len_utf8()
                } else {
                    0
                }
            }
            RegexValue::Clase(regex_class) => {
                for (i, c) in text.chars().enumerate() {
                    if regex_class.validar_caracter(c) {
                        return i + c.len_utf8();
                    }
                }
                0
            }
        }
    }

    /// Devuelve la longitud de la coincidencia del valor al inicio del texto.
    ///
    /// # Arguments
    ///
    /// * `value` - El texto en el que se va a buscar la coincidencia del valor.
    ///
    /// # Returns
    ///
    /// Devuelve la longitud de la coincidencia del valor al inicio del texto.
    ///
    /// Si no se encuentra ninguna coincidencia al inicio del texto, devuelve 0.
    pub fn is_same(&self, value: &str) -> usize {
        match self {
            RegexValue::Literal(c) => {
                if value.starts_with(*c) {
                    c.len_utf8()
                } else {
                    0
                }
            }
            RegexValue::Wildcard => {
                if let Some(next_char) = value.chars().next() {
                    next_char.len_utf8()
                } else {
                    0
                }
            }
            RegexValue::Clase(clase) => {
                if let Some(c) = value.chars().next() {
                    if clase.validar_caracter(c) {
                        c.len_utf8()
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_literal() {
        let value = RegexValue::Literal('a');
        assert_eq!(value.matches("abc"), 1);
        assert_eq!(value.matches("123"), 0);
    }

    #[test]
    fn test_matches_wildcard() {
        let value = RegexValue::Wildcard;
        assert_eq!(value.matches("abc"), 1);
        assert_eq!(value.matches("123"), 1);
        assert_eq!(value.matches(""), 0);
    }

    #[test]
    fn test_matches_clase() {
        let clase = RegexClase::Alpha;
        let value = RegexValue::Clase(clase.clone());
        assert_eq!(value.matches("mati"), 1);
        assert_eq!(value.matches("2001"), 0);

        let clase_custom = RegexClase::Custom(vec!['m', 'a', 't'], false);
        let value_custom = RegexValue::Clase(clase_custom.clone());
        assert_eq!(value_custom.matches("mat"), 1);
        assert_eq!(value_custom.matches("123"), 0);
    }

    #[test]
    fn test_is_same_literal() {
        let value = RegexValue::Literal('m');
        assert_eq!(value.is_same("mati"), 1);
        assert_eq!(value.is_same("123"), 0);
    }

    #[test]
    fn test_is_same_wildcard() {
        let value = RegexValue::Wildcard;
        assert_eq!(value.is_same("mati"), 1);
        assert_eq!(value.is_same("2001"), 1);
        assert_eq!(value.is_same(""), 0);
    }

    #[test]
    fn test_is_same_clase() {
        let clase = RegexClase::Alpha;
        let value = RegexValue::Clase(clase.clone());
        assert_eq!(value.is_same("abc"), 1);
        assert_eq!(value.is_same("123"), 0);

        let clase_custom = RegexClase::Custom(vec!['a', 'b', 'c'], false);
        let value_custom = RegexValue::Clase(clase_custom.clone());
        assert_eq!(value_custom.is_same("abc"), 1);
        assert_eq!(value_custom.is_same("123"), 0);
    }
}
