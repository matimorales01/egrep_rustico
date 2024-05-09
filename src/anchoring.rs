use crate::{regex_step::RegexStep, regex_value::RegexValue};

/// Estructura que representa el anclaje de la expresión regular al inicio y/o final de la cadena.
#[derive(Clone, Debug)]
pub struct Anchoring {
    anchoring_start: bool,
    anchoring_end: bool,
}

impl Default for Anchoring {
    /// Crea una nueva instancia de `Anchoring` con ambos anclajes desactivados por defecto.
    fn default() -> Self {
        Self::new()
    }
}

impl Anchoring {
    /// Crea una nueva instancia de `Anchoring` con ambos anclajes desactivados.
    pub fn new() -> Anchoring {
        Anchoring {
            anchoring_start: false,
            anchoring_end: false,
        }
    }

    /// Actualiza el estado de los anclajes en base al carácter actual.
    ///
    /// # Arguments
    ///
    /// * `current_char` - El carácter actual que se está procesando en la expresión regular.
    pub fn update_anchoring(&mut self, current_char: char) {
        match current_char {
            '^' => self.anchoring_start = true,
            '$' => self.anchoring_end = true,
            _ => {
                self.anchoring_start = false;
                self.anchoring_end = false;
            }
        }
    }

    /// Verifica si la cadena coincide con el patrón de la expresión regular con respecto a los anclajes.
    ///
    /// # Arguments
    ///
    /// * `steps` - Los pasos de la expresión regular.
    /// * `value` - La cadena que se está evaluando.
    ///
    /// # Returns
    ///
    /// `true` si la cadena coincide con el patrón con respecto a los anclajes, de lo contrario `false`.
    pub fn match_anchor(&self, steps: &[RegexStep], value: &str) -> bool {
        match (self.anchoring_start, self.anchoring_end) {
            (true, false) => {
                if !steps.is_empty() {
                    let pattern = Self::steps_to_string(steps);
                    if let Some(first_step) = steps.first() {
                        if let RegexValue::Literal(first_char) = &first_step.val {
                            return value.starts_with(*first_char)
                                && value[1..].starts_with(&pattern[1..]);
                        }
                    }
                }
                false
            }
            (false, true) => {
                if !steps.is_empty() {
                    let pattern = Self::steps_to_string(steps);
                    return value.ends_with(&pattern);
                }
                false
            }
            _ => false,
        }
    }

    /// Convierte los pasos de la expresión regular en una cadena.
    ///
    /// # Arguments
    ///
    /// * `steps` - Los pasos de la expresión regular.
    ///
    /// # Returns
    ///
    /// Una cadena que representa los pasos de la expresión regular.
    fn steps_to_string(steps: &[RegexStep]) -> String {
        steps
            .iter()
            .map(|step| match &step.val {
                RegexValue::Literal(c) => c.to_string(),
                RegexValue::Wildcard => ".".to_string(),
                RegexValue::Clase(_) => "".to_string(),
            })
            .collect()
    }
}
