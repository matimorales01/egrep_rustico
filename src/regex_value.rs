use crate::regex_clase::RegexClase;

#[derive(Debug, Clone)]
pub enum RegexValue {
    Literal(char),
    Wildcard, // comodin
    Clase(RegexClase),
}

impl RegexValue {
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
                    next_char.len_utf8() // Devuelve la longitud del primer carácter en la cadena
                } else {
                    0 // No hay más caracteres en la cadena
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
                    0 // No hay más caracteres en la cadena
                }
            }
        }
    }
    pub fn is_same_from_end(&self, text: &str) -> usize {
        match self {
            RegexValue::Literal(c) => {
                if text.ends_with(*c) {
                    c.len_utf8()
                } else {
                    0
                }
            }
            RegexValue::Wildcard => {
                if let Some(c) = text.chars().last() {
                    c.len_utf8()
                } else {
                    0
                }
            }
            RegexValue::Clase(regex_class) => {
                if let Some(c) = text.chars().last() {
                    if regex_class.validar_caracter(c) {
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
