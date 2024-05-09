use std::collections::VecDeque;
use std::str::Chars;

use crate::{
    anchoring::Anchoring, evaluated_step::EvaluatedStep, grep_error::GrepError,
    regex_clase::RegexClase, regex_rep::RegexRep, regex_step::RegexStep, regex_value::RegexValue,
};

#[derive(Debug, Clone)]
pub struct Regex {
    steps: Vec<RegexStep>,
    anchoring_start: bool,
    anchoring_end: bool,
    anchoring: Anchoring,
}

impl Regex {
    pub fn new(expression: &str) -> Result<Self, GrepError> {
        let mut steps: Vec<RegexStep> = vec![];

        let mut chars_iter = expression.chars();

        let mut anchoring_start = false;

        let mut anchoring_end = false;

        let mut anchoring = Anchoring::new();

        while let Some(c) = chars_iter.next() {
            let step = match c {
                '.' => Some(RegexStep {
                    rep: RegexRep::Exact(1),
                    val: RegexValue::Wildcard,
                }),
                'a'..='z' | 'A'..='Z' | '0'..='9' | ' ' => Some(RegexStep {
                    rep: RegexRep::Exact(1),
                    val: RegexValue::Literal(c),
                }),
                '*' => {
                    if let Some(last) = steps.last_mut() {
                        last.rep = RegexRep::Any;
                    } else {
                        return Err(GrepError::Err);
                    }

                    None
                }
                '^' => {
                    if steps.is_empty() {
                        anchoring_start = true;
                        anchoring.update_anchoring('^');
                    } else {
                        return Err(GrepError::Err);
                    }
                    None
                }
                '$' => {
                    if chars_iter.clone().next().is_none() {
                        anchoring_end = true;
                        anchoring.update_anchoring('$');
                    } else {
                        return Err(GrepError::Err);
                    }
                    None
                }
                '+' => {
                    if let Some(last) = steps.last_mut() {
                        last.rep = RegexRep::Range {
                            min: Some(1),
                            max: None,
                        }
                    } else {
                        return Err(GrepError::Err);
                    }
                    None
                }

                '?' => {
                    if let Some(last) = steps.last_mut() {
                        last.rep = RegexRep::Range {
                            min: Some(0),
                            max: Some(1),
                        };
                    } else {
                        return Err(GrepError::Err);
                    }
                    None
                }

                '{' => {
                    let mut min = String::new();
                    let mut max = String::new();
                    let mut mode = 0; // 0 = leyendo mínimo, 1 = leyendo máximo

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
                    None
                }

                '[' => {
                    if chars_iter.clone().next() == Some('[') {
                        // Handle character class
                        let class_content = Self::read_character_class(&mut chars_iter)?;
                        Some(RegexStep {
                            rep: RegexRep::Exact(1),
                            val: class_content,
                        })
                    } else {
                        // Handle bracket expression
                        let bracket_content = Self::read_bracket_expression(&mut chars_iter)?;
                        Some(RegexStep {
                            rep: RegexRep::Exact(1),
                            val: bracket_content,
                        })
                    }
                }

                '\\' => match chars_iter.next() {
                    Some(literal) => Some(RegexStep {
                        rep: RegexRep::Exact(1),
                        val: RegexValue::Literal(literal),
                    }),
                    None => return Err(GrepError::Err),
                },
                _ => return Err(GrepError::Err),
            };

            if let Some(p) = step {
                steps.push(p);
            }
        }

        Ok(Regex {
            steps,
            anchoring_end,
            anchoring_start,
            anchoring,
        })
    }

    pub fn test(&self, value: &str) -> Result<bool, GrepError> {
        if !value.is_ascii() {
            return Err(GrepError::Err);
        }

        let mut index = 0;

        let mut queue = VecDeque::from(self.steps.clone());
        let mut stack: Vec<EvaluatedStep> = Vec::new();
        if self.anchoring_end {
            if self.anchoring.match_anchor(&self.steps, value) {
                return Ok(true);
            } else {
                return Ok(false);
            }
        }
        'steps: while let Some(step) = queue.pop_front() {
            match step.rep {
                RegexRep::Exact(n) => {
                    let mut match_size = 0;
                    for _ in 0..n {
                        let size = if !self.anchoring_start && index == 0 {
                            step.val.matches(&value[index..])
                        } else {
                            step.val.is_same(&value[index..])
                        };
                        if size == 0 {
                            match EvaluatedStep::backtrack(step.clone(), &mut stack, &mut queue) {
                                Some(size) => {
                                    index -= size;
                                    continue 'steps;
                                }
                                None => return Ok(false),
                            }
                        } else {
                            match_size += size;
                            index += size;
                        }
                    }
                    stack.push(EvaluatedStep {
                        step: step.clone(),
                        match_size,
                        backtrackeable: false,
                    });
                }
                RegexRep::Any => {
                    let mut keep_matching = true;
                    while keep_matching {
                        let match_size = if !self.anchoring_start && index == 0 {
                            step.val.matches(&value[index..])
                        } else {
                            step.val.is_same(&value[index..])
                        };
                        if match_size != 0 {
                            index += match_size;
                            stack.push(EvaluatedStep {
                                step: step.clone(),
                                match_size,
                                backtrackeable: true,
                            });
                        } else {
                            keep_matching = false;
                        }
                    }
                }
                RegexRep::OneOrMore => {
                    let mut match_size = 0;
                    loop {
                        let size = step.val.matches(&value[index..]);
                        if size == 0 {
                            break;
                        }
                        match_size += size;
                        index += size;
                    }
                    if match_size == 0 {
                        return Ok(false);
                    }
                    stack.push(EvaluatedStep {
                        step: step.clone(),
                        match_size,
                        backtrackeable: true,
                    });
                }
                RegexRep::Range { min, max } => {
                    let mut count = 0;
                    let mut match_size = 0;
                    while count
                        < match max {
                            Some(value) => value,
                            None => usize::MAX,
                        }
                    {
                        let size = if !self.anchoring_start && index == 0 {
                            step.val.matches(&value[index..])
                        } else {
                            step.val.is_same(&value[index..])
                        };
                        if size == 0 {
                            break;
                        }
                        index += size;
                        count += 1;
                        match_size += size;
                    }
                    if let Some(min_value) = min {
                        if count < min_value {
                            return Ok(false);
                        }
                    }
                    stack.push(EvaluatedStep {
                        step: step.clone(),
                        match_size,
                        backtrackeable: true,
                    });
                }
            }
        }

        if self.anchoring_start && self.anchoring.match_anchor(&self.steps, value) {
            return Ok(true);
        }

        Ok(true)
    }

    pub fn crear_regex(regular_expression: &str) -> Result<Vec<Regex>, GrepError> {
        let mut regex_vec: Vec<Regex> = Vec::new();

        for subexpression in regular_expression.split('|') {
            if !subexpression.is_empty() {
                let regex = Regex::new(subexpression)?;
                regex_vec.push(regex);
            }
        }

        Ok(regex_vec)
    }

    fn read_character_class(chars_iter: &mut Chars) -> Result<RegexValue, GrepError> {
        let mut class_content = String::new();
        for char_in_class in chars_iter.by_ref() {
            if char_in_class == ']' {
                class_content.push(char_in_class);
                break;
            }
            class_content.push(char_in_class);
        }

        // Leer el corchete de cierre ']' adicional
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
                println!("Unrecognized character class: {:?}", class_content);
                return Err(GrepError::Err);
            }
        };

        Ok(RegexValue::Clase(regex_clase))
    }

    fn read_bracket_expression(chars_iter: &mut Chars) -> Result<RegexValue, GrepError> {
        let mut characters = String::new();
        let mut negated = false;
        // Verificar si el conjunto de caracteres es negado
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
            RegexClase::Custom("".chars().collect(), negated)
        } else {
            RegexClase::Custom(characters.chars().collect(), negated)
        };
        Ok(RegexValue::Clase(clase))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match0() -> Result<(), GrepError> {
        let value = "abcdef";
        let regex = Regex::new("abcd")?;
        let matches: bool = regex.test(value)?;
        assert_eq!(matches, true);

        Ok(())
    }

    #[test]
    fn test_match() -> Result<(), GrepError> {
        let value = "abcdef";
        let regex = Regex::new("ab.*e")?;
        let matches = regex.test(value)?;
        assert_eq!(matches, true);

        Ok(())
    }

    #[test]
    fn test_no_match0() -> Result<(), GrepError> {
        let value = "abcdef";
        let regex = Regex::new("aaaaaa")?;
        let matches = regex.test(value)?;
        assert_eq!(matches, false);

        Ok(())
    }

    #[test]
    fn test_no_match() -> Result<(), GrepError> {
        let value = "abcdef";
        let regex = Regex::new("ab.*h")?;
        let matches = regex.test(value)?;
        assert_eq!(matches, false);

        Ok(())
    }

    #[test]
    fn test_match2() -> Result<(), GrepError> {
        let value = "ab1234cdefg";
        let regex = Regex::new("ab.*c.*f")?;
        let matches = regex.test(value)?;
        assert_eq!(matches, true);

        Ok(())
    }

    #[test]
    fn test_no_match2() -> Result<(), GrepError> {
        let value = "ab1234cdegh";
        let regex = Regex::new("ab.*c.*f")?;
        let matches = regex.test(value)?;
        assert_eq!(matches, false);

        Ok(())
    }
    #[test]
    fn test_es_digit() -> Result<(), GrepError> {
        let value = "1 es un numero";
        let regex = Regex::new("[[:digit:]]")?;

        let matches = regex.test(value)?;
        println!("Resultado de la expresión regular: {}", matches);
        assert_eq!(matches, true);
        Ok(())
    }
    #[test]
    fn test_este_o_este() -> Result<(), GrepError> {
        let value = "apple";
        let regex = Regex::new("a?e")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, true);
        Ok(())
    }
    #[test]
    fn test_este_o_este_false() -> Result<(), GrepError> {
        let value = "bokit";
        let regex = Regex::new("a?e")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, false);
        Ok(())
    }
    #[test]
    fn test_curvy_bracket() -> Result<(), GrepError> {
        let value = "maaaaati";
        let regex = Regex::new("ma{5,6}ti")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, true);
        Ok(())
    }
    #[test]
    fn test_curvy_bracket_false() -> Result<(), GrepError> {
        let value = "mati";
        let regex = Regex::new("ma{5,6}ti")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, false);
        Ok(())
    }
    #[test]
    fn test_bracket() -> Result<(), GrepError> {
        let value = "la a es una vocal";
        let regex = Regex::new("la [aeiou] es una vocal")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, true);
        Ok(())
    }
    #[test]
    fn test_bracket_negada() -> Result<(), GrepError> {
        let value = "la z no es una vocal";
        let regex = Regex::new("la [^aeiou] no es una vocal")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, true);
        Ok(())
    }
    #[test]
    fn test_bracket_or() -> Result<(), GrepError> {
        let value = "abd";
        let regex = Regex::new("a[bc]d")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, true);
        Ok(())
    }
    #[test]
    fn test_bracket_or_false() -> Result<(), GrepError> {
        let value = "ald";
        let regex = Regex::new("a[bc]d")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, false);
        Ok(())
    }

    #[test]
    fn test_character_space() -> Result<(), GrepError> {
        let value = "hola mundo";
        let regex = Regex::new("hola[[:space:]]mundo")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, true);
        Ok(())
    }
    #[test]
    fn test_character_space_false() -> Result<(), GrepError> {
        let value = "holamundo";
        let regex = Regex::new("hola[[:space:]]mundo")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, false);
        Ok(())
    }

    #[test]
    fn test_character_alnum() -> Result<(), GrepError> {
        let value = "el caracter a no es un simbolo";
        let regex = Regex::new("el caracter [[:alnum:]] no es un simbolo")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, true);
        Ok(())
    }
    #[test]
    fn test_character_alnum_false() -> Result<(), GrepError> {
        let value = "el caracter $ no es un simbolo";
        let regex = Regex::new("el caracter [[:alnum:]] no es un simbolo")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, false);
        Ok(())
    }
    #[test]
    fn test_catedra_1() -> Result<(), GrepError> {
        let value_0 = "start with start";
        let value = "starting";
        let value_1 = "not start with start";
        let value_2 = "end with end";
        let value_3 = "only this line";
        let regex = Regex::new("^start")?;

        let matches = regex.clone().test(value)?;
        let matches_0 = regex.clone().test(value_0)?;
        let matches_1 = regex.clone().test(value_1)?;
        let matches_2 = regex.clone().test(value_2)?;
        let matches_3 = regex.clone().test(value_3)?;

        assert_eq!(matches, true);
        assert_eq!(matches_0, true);
        assert_eq!(matches_1, false);
        assert_eq!(matches_2, false);
        assert_eq!(matches_3, false);

        Ok(())
    }
    #[test]
    fn test_catedra_2() -> Result<(), GrepError> {
        let value_0 = "abcd";
        let value = "abcdd";
        let value_1 = "abd";
        let value_2 = "hola abcd chau";
        let value_3 = "abhhd";
        let regex = Regex::new("ab.?d")?;

        let matches = regex.clone().test(value)?;
        let matches_0 = regex.clone().test(value_0)?;
        let matches_1 = regex.clone().test(value_1)?;
        let matches_2 = regex.clone().test(value_2)?;
        let matches_3 = regex.clone().test(value_3)?;

        assert_eq!(matches, true);
        assert_eq!(matches_0, true);
        assert_eq!(matches_1, true);
        assert_eq!(matches_2, false);
        assert_eq!(matches_3, false);

        Ok(())
    }
    #[test]
    fn test_caracteres_alfabetico() -> Result<(), GrepError> {
        let regex = Regex::new("matias");

        assert_eq!(regex.unwrap().test("matias")?, true);
        Ok(())
    }
    #[test]
    fn test_muchos_caracteres_alfabeticos() -> Result<(), GrepError> {
        let regex = Regex::new("z");
        let regex1 = Regex::new("ele");
        let regex2 = Regex::new("e");
        let regex3 = Regex::new("ue");
        let regex4 = Regex::new("a");
        assert_eq!(regex.unwrap().test("ezequiel")?, true);
        assert_eq!(regex1.unwrap().test("elefante")?, true);
        assert_eq!(regex2.unwrap().test("el")?, true);
        assert_eq!(regex3.unwrap().test("aquel")?, true);
        assert_eq!(regex4.unwrap().test("no")?, false);
        Ok(())
    }
    #[test]
    fn test_caracteres_alfabetico_no_match() -> Result<(), GrepError> {
        let regex = Regex::new("cars");

        assert_eq!(regex.unwrap().test("matias")?, false);
        Ok(())
    }
    #[test]
    fn test_muchos_caracteres_alfabeticos_no_match() -> Result<(), GrepError> {
        let regex = Regex::new("y");
        let regex1 = Regex::new("z");
        let regex2 = Regex::new("aquel");
        let regex3 = Regex::new("mati");
        let regex4 = Regex::new("si");
        assert_eq!(regex.unwrap().test("ezequiel")?, false);
        assert_eq!(regex1.unwrap().test("elefante")?, false);
        assert_eq!(regex2.unwrap().test("el")?, false);
        assert_eq!(regex3.unwrap().test("aquel")?, false);
        assert_eq!(regex4.unwrap().test("no")?, false);
        Ok(())
    }

    #[test]
    fn test_caracteres_con_punto() -> Result<(), GrepError> {
        let regex = Regex::new("mati.morales");

        assert_eq!(regex.unwrap().test("matiamorales")?, true);
        Ok(())
    }

    #[test]
    fn test_caracteres_y_varios_puntos() -> Result<(), GrepError> {
        let regex = Regex::new("mati......s");

        assert_eq!(regex.unwrap().test("matimorales")?, true);
        Ok(())
    }

    #[test]
    fn test_caraceres_y_puntos_falso() -> Result<(), GrepError> {
        let regex = Regex::new("mat.as");

        assert_eq!(regex.unwrap().test("matas")?, false);
        Ok(())
    }
    #[test]
    fn test_signo_de_pregunta() -> Result<(), GrepError> {
        let regex = Regex::new("a?e");

        assert_eq!(regex.unwrap().test("apple")?, true);
        Ok(())
    }

    #[test]
    fn test_simbolo_suma_true() -> Result<(), GrepError> {
        let regex = Regex::new("matias");

        assert_eq!(regex.unwrap().test("ma+ti")?, true);
        Ok(())
    }

    #[test]
    fn test_bracket_expression_true() -> Result<(), GrepError> {
        let regex = Regex::new("ma[ti]as");
        assert_eq!(regex.unwrap().test("matas")?, true);
        Ok(())
    }

    #[test]
    fn test_bracket_expression_false() -> Result<(), GrepError> {
        let regex = Regex::new("ma[ti]as");
        assert_eq!(regex.unwrap().test("matias")?, false);
        Ok(())
    }

    #[test]
    fn test_bracket_expression_c_exacta() -> Result<(), GrepError> {
        let regex = Regex::new("mat{4}i");

        assert_eq!(regex.unwrap().test("jmatttti")?, true);
        Ok(())
    }
    #[test]
    fn test_bracket_expression_c_falso() -> Result<(), GrepError> {
        let regex = Regex::new("ma{5,6}ti");

        assert_eq!(regex.unwrap().test("maaaati")?, false);
        Ok(())
    }

    #[test]
    fn test_es_digit2() -> Result<(), GrepError> {
        let regex = Regex::new("[[:digit:]] es un numero");

        assert_eq!(regex.unwrap().test("9 es un numero")?, true);
        Ok(())
    }
    #[test]
    fn test_no_es_digit() -> Result<(), GrepError> {
        let regex = Regex::new("[[:digit:]] es un numero");

        assert_eq!(regex.unwrap().test("a es un numero")?, false);
        Ok(())
    }

    #[test]
    fn test_es_letra() -> Result<(), GrepError> {
        let regex = Regex::new("[[:alpha:]] es una letra");
        assert_eq!(regex.unwrap().test("e es una letra")?, true);
        Ok(())
    }
    #[test]
    fn test_no_es_letra() -> Result<(), GrepError> {
        let regex = Regex::new("[[:alpha:]] es una letra");
        assert_eq!(regex.unwrap().test("4 es una letra")?, false);
        Ok(())
    }
    #[test]
    fn test_punto_asterisco() -> Result<(), GrepError> {
        let regex = Regex::new("ab.*cd");

        assert_eq!(regex.unwrap().test("abholajajajajacd")?, true);
        Ok(())
    }

    #[test]
    fn test_signo_pesos() -> Result<(), GrepError> {
        let regex = Regex::new("abc$");
        assert_eq!(regex.unwrap().test("matiabc")?, true);
        Ok(())
    }
}
