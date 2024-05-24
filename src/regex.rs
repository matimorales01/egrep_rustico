use std::collections::VecDeque;

use crate::{
    anchoring::Anchoring, bracket_expression::BracketExpression, character_class::CharacterClass,
    evaluated_step::EvaluatedStep, grep_error::GrepError, regex_rep::RegexRep,
    regex_step::RegexStep, regex_value::RegexValue,
};

#[derive(Debug, Clone)]
pub struct Regex {
    steps: Vec<RegexStep>,
    anchoring: Anchoring,
}

impl Regex {
    pub fn new(expression: &str) -> Result<Self, GrepError> {
        let mut steps: Vec<RegexStep> = vec![];
        let mut chars_iter = expression.chars();
        let mut anchoring = Anchoring::new();

        while let Some(c) = chars_iter.next() {
            println!("Procesando carácter: {}", c);
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
                        println!("Encontrado '*', modificando el último paso: {:?}", last);
                        last.rep = RegexRep::Any;
                    } else {
                        return Err(GrepError::Err);
                    }
                    None
                }
                '^' => {
                    if steps.is_empty() {
                        anchoring.update_anchoring('^');
                    } else {
                        return Err(GrepError::Err);
                    }
                    None
                }
                '$' => {
                    if chars_iter.clone().next().is_none() {
                        anchoring.update_anchoring('$');
                    } else {
                        return Err(GrepError::Err);
                    }
                    None
                }
                '+' => {
                    if let Some(last) = steps.last_mut() {
                        println!("Encontrado '+', modificando el último paso: {:?}", last);
                        match last.rep {
                            RegexRep::Exact(n) => {
                                last.rep = RegexRep::Range {
                                    min: Some(n),
                                    max: None,
                                };
                            }
                            RegexRep::Range { min, max } => {
                                if let Some(mut min_value) = min {
                                    min_value += 1;
                                    last.rep = RegexRep::Range {
                                        min: Some(min_value),
                                        max,
                                    };
                                } else {
                                    last.rep = RegexRep::Range { min: Some(1), max };
                                }
                            }
                            _ => {}
                        }
                    } else {
                        return Err(GrepError::Err);
                    }
                    None
                }
                '?' => {
                    if let Some(last) = steps.last_mut() {
                        println!("Encontrado '?', modificando el último paso: {:?}", last);
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
                    println!("Procesando  con BracketExpression::read_bracket_expression_c");
                    BracketExpression::read_bracket_expression_c(&mut chars_iter, &mut steps)?;
                    None
                }
                '[' => {
                    println!("Procesando '['");
                    if chars_iter.clone().next() == Some('[') {
                        let class_content = CharacterClass::read_character_class(&mut chars_iter)?;
                        Some(RegexStep {
                            rep: RegexRep::Exact(1),
                            val: class_content,
                        })
                    } else {
                        let bracket_content =
                            BracketExpression::read_bracket_expression(&mut chars_iter)?;
                        Some(RegexStep {
                            rep: RegexRep::Exact(1),
                            val: bracket_content,
                        })
                    }
                }
                '\\' => {
                    if let Some(special_char) = chars_iter.next() {
                        Some(RegexStep {
                            rep: RegexRep::Exact(1),
                            val: RegexValue::Literal(special_char),
                        })
                    } else {
                        return Err(GrepError::Err);
                    }
                }
                _ => return Err(GrepError::Err),
            };

            if let Some(p) = step {
                steps.push(p);
            }
        }

        println!("Creación de Regex completada con steps: {:?} y anchoring: {:?}", steps, anchoring);
        Ok(Regex { steps, anchoring })
    }

    pub fn test(&self, value: &str) -> Result<bool, GrepError> {
        if !value.is_ascii() {
            return Err(GrepError::Err);
        }
    
        let mut index = 0;
        let  queue = VecDeque::from(self.steps.clone());
    
        if self.anchoring.get_anchoring_end() {
            if self.anchoring.matches_anchoring(&self.steps, value) {
                return Ok(true);
            } else {
                return Ok(false);
            }
        }
        if self.anchoring.get_anchoring_start(){
            if self.anchoring.matches_anchoring(&self.steps, value) {
                return Ok(true);
            } else {
                return Ok(false);
            }
        }
    
        while index < value.len() {
            let mut local_index = index;
            let mut local_queue = queue.clone();
            let mut local_stack = Vec::new();
    
            'steps: while let Some(step) = local_queue.pop_front() {
                match step.rep {
                    RegexRep::Exact(n) => {
                        let mut match_size = 0;
                        for _ in 0..n {
                            let size = if local_index == 0 && !self.anchoring.get_anchoring_start() {
                                step.val.matches(&value[local_index..])
                            } else {
                                step.val.is_same(&value[local_index..])
                            };
                            if size == 0 {
                                match EvaluatedStep::backtrack(step.clone(), &mut local_stack, &mut local_queue) {
                                    Some(size) => {
                                        local_index -= size;
                                        continue 'steps;
                                    }
                                    None => break 'steps,
                                }
                            } else {
                                match_size += size;
                                local_index += size;
                            }
                        }
                        local_stack.push(EvaluatedStep {
                            step: step.clone(),
                            match_size,
                            backtrackeable: false,
                        });
                    }
                    RegexRep::Any => {
                        let mut keep_matching = true;
                        while keep_matching {
                            let match_size = if local_index == 0 && !self.anchoring.get_anchoring_start() {
                                step.val.matches(&value[local_index..])
                            } else {
                                step.val.is_same(&value[local_index..])
                            };
                            if match_size != 0 {
                                local_index += match_size;
                                local_stack.push(EvaluatedStep {
                                    step: step.clone(),
                                    match_size,
                                    backtrackeable: true,
                                });
                            } else {
                                keep_matching = false;
                            }
                        }
                    }
                    RegexRep::Range { min, max } => {
                        let mut count = 0;
                        let mut match_size = 0;
                        while count < match max {
                            Some(value) => value,
                            None => usize::MAX,
                        } {
                            let size = if local_index == 0 && !self.anchoring.get_anchoring_start() {
                                step.val.matches(&value[local_index..])
                            } else {
                                step.val.is_same(&value[local_index..])
                            };
                            if size == 0 {
                                break;
                            }
                            local_index += size;
                            count += 1;
                            match_size += size;
                        }
                        if let Some(min_value) = min {
                            if count < min_value {
                                break 'steps;
                            }
                        }
                        local_stack.push(EvaluatedStep {
                            step: step.clone(),
                            match_size,
                            backtrackeable: true,
                        });
                    }
                }
            }
    
            if local_queue.is_empty() {
                return Ok(true);
            }
    
            index += 1;
        }
    
        Ok(false)
    }
    

    pub fn crear_regex(regular_expression: &str) -> Result<Vec<Regex>, GrepError> {
        let mut regex_vec: Vec<Regex> = Vec::new();

        for subexpression in regular_expression.split('|') {
            if !subexpression.is_empty() {
                println!("Creando Regex para subexpresión: {}", subexpression);
                let regex = Regex::new(subexpression)?;
                regex_vec.push(regex);
            }
        }

        println!("Regex creados: {:?}", regex_vec);
        Ok(regex_vec)
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
    fn test_match_wildcard() -> Result<(), GrepError> {
        let value = "mati";
        let regex = Regex::new("ma.i")?;
        let matches = regex.test(value)?;
        assert_eq!(matches, true);

        Ok(())
    }
    #[test]
    fn test_match_wildcards() -> Result<(), GrepError> {
        let value = "matttkkiiii";
        let regex = Regex::new("ma........i")?;
        let matches = regex.test(value)?;
        assert_eq!(matches, true);

        Ok(())
    }

    #[test]
    fn test_match_wildcard_false() -> Result<(), GrepError> {
        let value = "matti";
        let regex = Regex::new("ma.i")?;
        let matches = regex.test(value)?;
        assert_eq!(matches, false);

        Ok(())
    }

    #[test]
    fn test_match_wildcards_false() -> Result<(), GrepError> {
        let value = "matii";
        let regex = Regex::new("ma........i")?;
        let matches = regex.test(value)?;
        assert_eq!(matches, false);

        Ok(())
    }

    #[test]
    fn test_is_digit() -> Result<(), GrepError> {
        let value = "1 es un numero";
        let regex = Regex::new("[[:digit:]]")?;

        let matches = regex.test(value)?;
        println!("Resultado de la expresión regular: {}", matches);
        assert_eq!(matches, true);
        Ok(())
    }

    #[test]
    fn test_rep_question_sign() -> Result<(), GrepError> {
        let value = "apple";
        let regex = Regex::new("a?e")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, true);
        Ok(())
    }

    #[test]
    fn test_rep_question_sign_false() -> Result<(), GrepError> {
        let value = "bokit";
        let regex = Regex::new("a?e")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, false);
        Ok(())
    }

    #[test]
    fn test_bracket_expression_c() -> Result<(), GrepError> {
        let value = "maaaaati";
        let regex = Regex::new("ma{5,6}ti")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, true);
        Ok(())
    }

    #[test]
    fn test_bracket_expression_c_false() -> Result<(), GrepError> {
        let value = "mati";
        let regex = Regex::new("ma{5,6}ti")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, false);
        Ok(())
    }

    #[test]
    fn test_bracket_expression() -> Result<(), GrepError> {
        let value = "la a es una vocal";
        let regex = Regex::new("la [aeiou] es una vocal")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, true);
        Ok(())
    }

    #[test]
    fn test_bracket_expression_false() -> Result<(), GrepError> {
        let value = "la f es una vocal";
        let regex = Regex::new("la [aeiou] es una vocal")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, false);
        Ok(())
    }

    #[test]
    fn test_bracket_expression_negated() -> Result<(), GrepError> {
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
    fn test_anchoring_start() -> Result<(), GrepError> {
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
    fn test_anchoring_start_false() -> Result<(), GrepError> {
        let value = "aguante bokita";
        let regex = Regex::new("^bokita")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, false);
        Ok(())
    }
    #[test]
    fn test_anchoring_end() -> Result<(), GrepError> {
        let value = "aguante bokita";
        let regex = Regex::new("bokita$")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, true);
        Ok(())
    }
    #[test]
    fn test_anchoring_end_false() -> Result<(), GrepError> {
        let value = "matiassss";
        let regex = Regex::new("bokita$")?;

        let matches = regex.test(value)?;
        assert_eq!(matches, false);
        Ok(())
    }
   
    #[test]
fn test_catedra_uno() -> Result<(), GrepError> {
    let value_0 = "abcd";
    let value = "abcdd";
    let value_1 = "abccd";
    let value_2 = "hola abcd chau";
    let regex = Regex::new("ab.d")?;

    let matches = regex.clone().test(value)?;
    let matches_0 = regex.clone().test(value_0)?;
    let matches_1 = regex.clone().test(value_1)?;
    let matches_2 = regex.clone().test(value_2)?;

    assert_eq!(matches, true);
    assert_eq!(matches_0, true);
    assert_eq!(matches_1, false);
    assert_eq!(matches_2, true);

    Ok(())
}
#[test]
fn test_catedra_dos() -> Result<(), GrepError> {
    let value_0 = "absalolngopsgdehejsd";
    let value = "abcdd";
    let value_1 = "abd";
    let value_2 = "que tul abuelita dime tu";
    let value_3 = "hola abcd chau";
    let value_4 = "te vamos a bochar";

    let regex = Regex::new("ab.*d")?;

    let matches = regex.clone().test(value)?;
    let matches_0 = regex.clone().test(value_0)?;
    let matches_1 = regex.clone().test(value_1)?;
    let matches_2 = regex.clone().test(value_2)?;
    let matches_3 = regex.clone().test(value_3)?;
    let matches_4 = regex.clone().test(value_4)?;



    assert_eq!(matches, true);
    assert_eq!(matches_0, true);
    assert_eq!(matches_1, true);
    assert_eq!(matches_2, true);
    assert_eq!(matches_3, true);
    assert_eq!(matches_4, false);



    Ok(())
}
#[test]
fn test_catedra_tres() -> Result<(), GrepError> {
    let value_0 = "abcd";
    let value = "abcccd";
    let value_1 = "hola abcccd chau";


    let regex = Regex::new("abc{3}d")?;

    let matches = regex.clone().test(value)?;
    let matches_0 = regex.clone().test(value_0)?;
    let matches_1 = regex.clone().test(value_1)?;

    assert_eq!(matches_0, false);
    assert_eq!(matches, true);
    assert_eq!(matches_1, true);




    Ok(())
}
#[test]
fn test_catedra_cuatro() -> Result<(), GrepError> {
    let value_0 = "abcd abcd";
    let value = "abd abcccd abd";
    let value_1 = "abcccccccd abcd";
    let value_2 = "en medio abccd abd fin";


    let regex = Regex::new("abc{2,5}d abc{0,}d")?;

    let matches = regex.clone().test(value)?;
    let matches_0 = regex.clone().test(value_0)?;
    let matches_1 = regex.clone().test(value_1)?;
    let matches_2 = regex.clone().test(value_2)?;

    assert_eq!(matches, true);
    assert_eq!(matches_0, false);
    assert_eq!(matches_1, false);
    assert_eq!(matches_2, true);



    Ok(())
}
#[test]
fn test_catedra_cinco() -> Result<(), GrepError> {
    let value_0 = "abd";
    let value = "abc";
    let value_1 = "agd";
    let value_2 = "cami figura abd";
    
    let regex = Regex::new("a[bc]d")?;
    let matches_0 = regex.clone().test(value_0)?;
    let matches = regex.clone().test(value)?;
    let matches_1 = regex.clone().test(value_1)?;
    let matches_2 = regex.clone().test(value_2)?;

    assert_eq!(matches_0, true);
    assert_eq!(matches, false);
    assert_eq!(matches_1, false);
    assert_eq!(matches_2, true);
    Ok(())
    }
    #[test]
    fn test_catedra_seis() -> Result<(), GrepError> {
        let value_0 = "abcd";
        let value = "abd";
        let value_1 = "abcccd";
        let value_2 = "hola abcd chau";

        
        let regex = Regex::new("abc+d")?;
        let matches_0 = regex.clone().test(value_0)?;
        let matches = regex.clone().test(value)?;
        let matches_1 = regex.clone().test(value_1)?;
        let matches_2 = regex.clone().test(value_2)?;
    
        assert_eq!(matches_0, true);
        assert_eq!(matches, false);
        assert_eq!(matches_1, true);
        assert_eq!(matches_2, true);
        Ok(())
        }
        #[test]
        fn test_catedra_siete() -> Result<(), GrepError> {
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
            assert_eq!(matches_2, true);
            assert_eq!(matches_3, false);
    
            Ok(())
        }
        #[test]
fn test_apple_or_melon() -> Result<(), GrepError> {
    let input = "banana\napple\norange\npineapple\nsoy melon\nen el medio watermelon va";
    let regexes = Regex::crear_regex("apple|melon")?;
    let mut expected_output = String::new();

    for regex in regexes {
        let mut matched_lines = String::new();
        for line in input.lines() {
            if regex.test(line)? {
                matched_lines.push_str(line);
                matched_lines.push('\n');
            }
        }
        expected_output.push_str(&matched_lines);
    }

    let output = "apple\npineapple\nsoy melon\nen el medio watermelon va\n";

    assert_eq!(expected_output, output);

    Ok(())
}
#[test]
fn test_complex_regex() -> Result<(), GrepError> {
    let input = "abc?def\n123*456\n789+10\nhola?\nesta no tiene que estar\nesta tampoco";
    let regexes = Regex::crear_regex("abc\\?def|123\\*456|789\\+10")?;
    let mut expected_output = String::new();

    for regex in regexes {
        let mut matched_lines = String::new();
        for line in input.lines() {
            if regex.test(line)? {
                matched_lines.push_str(line);
                matched_lines.push('\n');
            }
        }
        expected_output.push_str(&matched_lines);
    }

    let output = "abc?def\n123*456\n789+10\n";

    assert_eq!(expected_output, output);

    Ok(())
}


}


