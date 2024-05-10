use crate::{
    character_class::CharacterClass, grep_error::GrepError, regex_rep::RegexRep,
    regex_step::RegexStep, regex_value::RegexValue,
};
use std::str::Chars;

/// Representa una expresión entre corchetes `[...]` o llaves `{...}` .
pub struct BracketExpression;

impl Default for BracketExpression {
    fn default() -> Self {
        Self::new()
    }
}

impl BracketExpression {
    /// Crea una nueva instancia de `BracketExpression`.
    pub fn new() -> Self {
        BracketExpression
    }

    /// Maneja el operador de llaves `{}` para definir repeticiones.
    ///
    /// # Arguments
    ///
    /// * `chars_iter` - Un iterador de caracteres que representa la expresión entre llaves.
    /// * `steps` - Un vector mutable de pasos de regex para actualizar la estructura de repetición.
    ///
    /// # Returns
    ///
    /// Devuelve `Ok(())` si la operación de manejo del operador de llaves se realiza con éxito.
    ///
    /// Devuelve `Err(GrepError)` si ocurre algún error durante el procesamiento del operador de llaves.
    pub fn read_bracket_expression_c(
        chars_iter: &mut Chars,
        steps: &mut [RegexStep],
    ) -> Result<(), GrepError> {
        let mut min = String::new();
        let mut max = String::new();
        let mut mode = 0;

        for c in chars_iter.by_ref() {
            match c {
                '0'..='9' => {
                    if mode == 0 {
                        min.push(c);
                    } else {
                        max.push(c);
                    }
                }
                ',' => {
                    mode = 1;
                }
                '}' => {
                    let min = min.parse::<usize>().map_err(|_| GrepError::Err)?;
                    let max = if max.is_empty() {
                        None
                    } else {
                        Some(max.parse::<usize>().map_err(|_| GrepError::Err)?)
                    };
                    let rep = RegexRep::Range {
                        min: Some(min),
                        max,
                    };
                    if let Some(last) = steps.last_mut() {
                        last.rep = rep;
                    } else {
                        return Err(GrepError::Err);
                    }
                    break;
                }
                _ => return Err(GrepError::Err),
            }
        }
        Ok(())
    }

    /// Lee y procesa una expresión entre corchetes `[...]` y devuelve su representación como `RegexValue`.
    ///
    /// # Arguments
    ///
    /// * `chars_iter` - Un iterador de caracteres que representa la expresión entre corchetes.
    ///
    /// # Returns
    ///
    /// Devuelve `Ok(RegexValue)` si la expresión entre corchetes se procesa correctamente
    /// y se devuelve su representación como `RegexValue`.
    ///
    /// Devuelve `Err(GrepError)` si ocurre algún error durante el procesamiento de la expresión entre corchetes.
    pub fn read_bracket_expression(chars_iter: &mut Chars) -> Result<RegexValue, GrepError> {
        let mut characters = String::new();
        let mut negated = false;

        if let Some('^') = chars_iter.clone().next() {
            chars_iter.next();
            negated = true;
        }

        for inner_c in chars_iter.by_ref() {
            if inner_c == ']' {
                break;
            }
            characters.push(inner_c);
        }

        let clase = if characters.is_empty() {
            CharacterClass::Custom("".chars().collect(), negated)
        } else {
            CharacterClass::Custom(characters.chars().collect(), negated)
        };

        Ok(RegexValue::Clase(clase))
    }
}
