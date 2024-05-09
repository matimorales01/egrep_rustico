use crate::regex_step::RegexStep;
use std::collections::VecDeque;

/// Representa un paso evaluado de la expresión regular, incluyendo información sobre la coincidencia, el tamaño de la coincidencia y si es retrocedible.
#[derive(Debug, Clone)]
pub struct EvaluatedStep {
    pub step: RegexStep,
    pub match_size: usize,
    pub backtrackeable: bool,
}

impl EvaluatedStep {
    /// Realiza un retroceso basado en el paso actual evaluado, actualizando la pila de evaluaciones y la cola de pasos siguientes.
    ///
    /// # Arguments
    ///
    /// * `current` - El paso actual de la expresión regular.
    /// * `evaluated` - Una referencia mutable a la pila de pasos evaluados.
    /// * `next` - Una referencia mutable a la cola de pasos siguientes.
    ///
    /// # Returns
    ///
    /// Un valor opcional que representa el tamaño del retroceso realizado.
    pub fn backtrack(
        current: RegexStep,
        evaluated: &mut Vec<EvaluatedStep>,
        next: &mut VecDeque<RegexStep>,
    ) -> Option<usize> {
        let mut back_size = 0;
        next.push_front(current);

        while let Some(e) = evaluated.pop() {
            back_size += e.match_size;
            if e.backtrackeable {
                return Some(back_size);
            } else {
                next.push_front(e.step);
            }
        }
        None
    }
}
