#[derive(Debug, Clone)]
pub enum RType {
    Ch(char),              // character
    Ccl(String, bool),     // character group, +ve/-ve
    Cgd,                   // character class digit
    Cgw,                   // character class alphanumeric
    Qplus(Box<RType>),     // match one ore more time for previous RType
    Qquestion(Box<RType>), // match zero or one time for previous RType
    Wildcard,              // match any character
    AltOr(Box<Vec<RType>>, Box<Vec<RType>>) // match (a|b), a or b
}

// NOTE: we'll be ignoring multi-line regex, so start/end anchor for newline is ignored read:
// https://learn.microsoft.com/en-us/dotnet/standard/base-types/anchors-in-regular-expressions#start-of-string-only-a
#[derive(Debug, Clone)]
pub enum StringAnchor {
    Start,
    End,
    None,
}

#[derive(Debug)]
pub struct RE {
    pub rtype: Vec<RType>,
    pub anchor: StringAnchor,
}

pub fn get_regex_pattern(pattern: &str) -> RE {
    let mut re_pattern: Vec<RType> = vec![];
    // let mut chiter: std::iter::Peekable<Chars>;
    let mut string_anchor = StringAnchor::None;
    let mut pattern = &pattern[..];
    if pattern.starts_with('^') {
        string_anchor = StringAnchor::Start;
        pattern = &pattern[1..];
    }
    if pattern.ends_with('$') {
        string_anchor = StringAnchor::End;
        pattern = &pattern[..pattern.len() - 1];
    }
    // chiter = pattern.chars().peekable();
    let cpattern = pattern.chars().collect::<Vec<_>>();
    let mut pidx: usize = 0; // pattern index
    let mut idx = 0; // character index
    'out: loop {
        if idx == cpattern.len() {
            break 'out RE {
                rtype: re_pattern,
                anchor: string_anchor,
            };
        }

        match cpattern[idx] {
            '+' | '?' if pidx >= 1 => {
                match re_pattern[pidx - 1] {
                    RType::Qplus(_) | RType::Qquestion(_) => {
                        panic!("Quantifier can't be applied to another quantifier")
                    }
                    _ => {}
                }
                if cpattern[idx] == '+' {
                    re_pattern[pidx - 1] = RType::Qplus(Box::new(re_pattern[pidx - 1].clone()));
                } else {
                    re_pattern[pidx - 1] = RType::Qquestion(Box::new(re_pattern[pidx - 1].clone()));
                }
                pidx -= 1;
            }
            '\\' => {
                // TODO: add support to match '\' too, currently this logic ignores it
                while '\\' == cpattern[idx+1] {
                    idx += 1;
                }
                idx += 1;
                re_pattern.push(match cpattern[idx] {
                    'd' => RType::Cgd,
                    'w' => RType::Cgw,
                    '+' => RType::Ch('+'),
                    _ => {
                        #[cfg(debug_assertions)]
                        println!("=> {}", cpattern[idx]);
                        unreachable!();
                    }
                });
            },
            '(' => {
                // get_regex_pattern(chiter.collect())
                let re_left = get_regex_pattern(&cpattern[idx+1..].iter().collect::<String>());
                let mut found = false;
                while idx < cpattern.len() {
                    if cpattern[idx] == '|' {
                        found = true;
                        break;
                    }
                    idx += 1;
                }
                if !found {
                    panic!("Invalid regex pattern provided. Missing | for alternation.");
                }
                let re_right = get_regex_pattern(&cpattern[idx+1..].iter().collect::<String>());
                found = false;
                while idx < cpattern.len() {
                    if cpattern[idx] == ')' {
                        found = true;
                        break;
                    }
                    idx += 1;
                }
                if !found {
                    panic!("Invalid regex pattern provided. Missing closing ) for opened (")
                }
                re_pattern.push(RType::AltOr(Box::new(re_left.rtype), Box::new(re_right.rtype)));
            },
            '|'|')' => {
                // should go back to '(' match block
                return RE {
                    rtype: re_pattern,
                    anchor: StringAnchor::None,
                };
            },
            '[' => {
                // let's assume that we'll find ']' later on always, for now
                let mut gmode = true;
                if idx+1 < cpattern.len() {
                    if cpattern[idx+1] == '^' {
                        gmode = false;
                        idx += 1;
                    }
                }
                let mut group = String::new();
                while idx < cpattern.len() {
                    if cpattern[idx] == ']' {
                        break;
                    }
                    group.push(cpattern[idx]);
                    idx += 1;
                }
                re_pattern.push(RType::Ccl(group, gmode));
            },
            '.' => re_pattern.push(RType::Wildcard),
            _ => re_pattern.push(RType::Ch(cpattern[idx])),
        };
        pidx += 1;
        idx += 1;
    }
}

fn match_quantifier(input_line: &str, re: &RE) -> usize {
    let mut idx: usize = 0;
    // NOTE: using match_here will not create cycle, because a quantifer will not have another
    // quantifier as RType
    assert_eq!(re.rtype.len(), 1);

    #[cfg(debug_assertions)]
    println!("+|? {:?}: {:?}", &input_line, &re);
    match &re.rtype[0] {
        RType::Qplus(rtype) | RType::Qquestion(rtype) => {
            let new_re = RE {
                rtype: vec![*rtype.clone()],
                anchor: StringAnchor::None,
            };
            #[cfg(debug_assertions)]
            println!("+|? rtype: {:?}, {:?}", re.rtype[0], &re);
            if let RType::Qplus(_) = re.rtype[0] {
                #[cfg(debug_assertions)]
                println!("+|? [if] new_re: {:?}", &new_re);
                while match_here(&input_line[idx..], &new_re).0 {
                    idx += 1;
                }
            } else if let RType::Qquestion(_) = re.rtype[0] {
                if match_here(&input_line[idx..], &new_re).0 {
                    idx += 1;
                }
            }
        }
        _ => {}
    }
    idx
}

fn match_here(input_line: &str, re_pattern: &RE) -> (bool, usize) {
    let input_chars = input_line.chars().collect::<Vec<_>>();
    let mut idx = 0;
    let rtype_iter = re_pattern.rtype.iter().peekable();
    #[cfg(debug_assertions)]
    println!("[here] {:?}: {:?}", &input_line, re_pattern);
    for rtype in rtype_iter {
        if idx == input_chars.len() {
            return (false, idx);
        }
        #[cfg(debug_assertions)]
        println!("[here for] {:?}: {:?}", &input_line, rtype);
        match rtype {
            RType::Qplus(_) | RType::Qquestion(_) => {
                #[cfg(debug_assertions)]
                println!("[here for] calling +|? for rtype: {:?}", rtype);
                let tidx = match_quantifier(
                    &input_line[idx..],
                    &RE {
                        rtype: vec![rtype.clone()],
                        anchor: StringAnchor::None, // match_quantifier doesn't need to know about StringAnchor
                    },
                );
                #[cfg(debug_assertions)]
                println!("[here for] +|? tidx: {}", tidx);
                if let RType::Qplus(_) = rtype {
                    if tidx == 0 {
                        return (false, idx);
                    }
                }
                if tidx == 0 && idx != 0 {
                    idx -= 1; // no match, no increase (idx += 1 will be done at the end)
                } else if tidx > 0 {
                    // there's no point in subtracting if there's no match
                    idx += tidx - 1;
                }
            },
            RType::AltOr(re_left, re_right) => {
                // NOTE: considering that the alternation will happen where len of rtype in re_left
                // and re_right is going to be same and no Qplus or Qquestion is going to be used
                let left_status = match_here(&input_line[idx..], &RE {
                    rtype: re_left.as_ref().clone(),
                    anchor: StringAnchor::None,
                });
                if !left_status.0 {
                    let right_status = match_here(&input_line[idx..], &RE {
                        rtype: re_right.as_ref().clone(),
                        anchor: StringAnchor::None,
                    });
                    if !right_status.0 {
                        return (false, idx + left_status.1);
                    }
                    if right_status.1 > 0 {
                        idx += right_status.1 - 1;
                    }
                } else {
                    if left_status.1 > 0 {
                        idx += left_status.1 - 1;
                    }
                }
            },
            RType::Ch(c) if &input_chars[idx] != c => {
                return (false, idx);
            }
            RType::Ccl(group, mode) => {
                if *mode {
                    let mut is_match = false;
                    for cg in group.chars() {
                        if cg == input_chars[idx] {
                            is_match = true;
                            break;
                        }
                    }
                    if !is_match {
                        return (false, idx);
                    }
                } else {
                    // for when mode is not true, will continue later
                    let mut is_match = true;
                    for cg in group.chars() {
                        if cg == input_chars[idx] {
                            is_match = false;
                            break;
                        }
                    }
                    if !is_match {
                        return (false, idx);
                    }
                }
            }
            RType::Cgd => {
                if !input_chars[idx].is_ascii_digit() {
                    return (false, idx);
                }
            }
            RType::Cgw => {
                if !input_chars[idx].is_ascii_alphanumeric() {
                    return (false, idx);
                }
            }
            RType::Wildcard => {} // do nothing
            _ => {}
        }
        idx += 1;
    }
    if idx < input_chars.len() {
        if let StringAnchor::End = re_pattern.anchor {
            return (false, idx);
        }
    }
    (true, idx)
}

pub fn match_pattern(input_line: &str, re: &RE) -> bool {
    if input_line.is_empty() {
        false
    } else if let StringAnchor::Start = re.anchor {
        match_here(input_line, &re).0
    } else {
        if match_pattern(&input_line[1..], re) {
            return true;
        }
        if !match_here(input_line, &re).0 {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod test {
    use crate::{get_regex_pattern, match_pattern, match_quantifier};

    #[test]
    fn quantifier_plus() {
        let re = get_regex_pattern("o+");
        assert_eq!(match_quantifier("oo", &re), 2);
    }

    #[test]
    fn full_pattern_quantifier_plus() {
        let re_pattern = get_regex_pattern("g+o+$");
        let input_line = "logs are good";
        assert_eq!(match_pattern(&input_line, &re_pattern), false);
        let re_pattern = get_regex_pattern("g+o+d$");
        assert_eq!(match_pattern(&input_line, &re_pattern), true);
    }

    #[test]
    fn full_pattern_quantifier_question() {
        let re_pattern = get_regex_pattern("g+l?o+d$");
        let input_line = "logs are good";
        assert_eq!(match_pattern(&input_line, &re_pattern), true);
        let re_pattern = get_regex_pattern("ca?t");
        let input_line = "cat";
        assert_eq!(match_pattern(&input_line, &re_pattern), true);
    }
}
