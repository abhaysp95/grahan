use std::str::Chars;

#[derive(Debug, Clone)]
pub enum RType {
    Ch(char),          // character
    Ccl(String, bool), // character group, +ve/-ve
    Cgd,               // character class digit
    Cgw,               // character class alphanumeric
    Qplus(Box<RType>), // match one ore more time for previous RType
    Qquestion(Box<RType>), // match zero or one time for previous RType
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
    let mut chiter: std::iter::Peekable<Chars>;
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
    chiter = pattern.chars().peekable();
    let mut pidx: usize = 0; // pattern index
    'out: loop {
        if chiter.peek().is_none() {
            break 'out RE {
                rtype: re_pattern,
                anchor: string_anchor,
            };
        }

        let c = chiter.next().unwrap();
        match c {
            '+'|'?' if pidx >= 1 => {
                match re_pattern[pidx-1] {
                    RType::Qplus(_)|RType::Qquestion(_) => {
                        panic!("Quantifier can't be applied to another quantifier")
                    },
                    _ => {},
                }
                if c == '+' {
                    re_pattern[pidx - 1] = RType::Qplus(Box::new(re_pattern[pidx - 1].clone()));
                } else {
                    re_pattern[pidx - 1] = RType::Qquestion(Box::new(re_pattern[pidx - 1].clone()));
                }
                pidx -= 1;
            }
            '\\' => {
                // NOTE: add support to match '\' too, currently this logic ignores it
                while let Some('\\') = chiter.peek() {
                    chiter.next();
                }
                re_pattern.push(match chiter.next().unwrap() {
                    'd' => RType::Cgd,
                    'w' => RType::Cgw,
                    '+' => RType::Ch('+'),
                    _ => {
                        #[cfg(debug_assertions)]
                        println!("=> {}", c);
                        unreachable!();
                    }
                });
            }
            '[' => {
                // let's assume that we'll find ']' later on always, for now
                let mut gmode = true;
                if let Some(mode) = chiter.peek() {
                    if mode == &'^' {
                        gmode = false;
                        chiter.next();
                    }
                }
                let mut group = String::new();
                while let Some(c) = chiter.next() {
                    if c == ']' {
                        break;
                    }
                    group.push(c);
                }
                re_pattern.push(RType::Ccl(group, gmode));
            }
            _ => re_pattern.push(RType::Ch(c)),
        };

        pidx += 1;
    }
}

fn match_quantifier(input_line: &str, re: &RE) -> usize {
    let mut idx: usize = 0;
    // NOTE: using match_here will not create cycle, because a quantifer will not have another
    // quantifier as RType
    assert_eq!(re.rtype.len(), 1);

    println!("+|? {:?}: {:?}", &input_line, &re);
    match &re.rtype[0] {
        RType::Qplus(rtype) | RType::Qquestion(rtype) => {
            let new_re = RE {
                rtype: vec![*rtype.clone()],
                anchor: StringAnchor::None,
            };
            println!("+|? rtype: {:?}, {:?}", re.rtype[0], &re);
            if let RType::Qplus(_) = re.rtype[0] {
                println!("+|? [if] new_re: {:?}", &new_re);
                while match_here(&input_line[idx..], &new_re) {
                    idx += 1;
                }
            } else if let RType::Qquestion(_) = re.rtype[0] {
                if match_here(&input_line[idx..], &new_re) {
                    idx += 1;
                }
            }
        },
        _ => {},
    }
    idx
}

fn match_here(input_line: &str, re_pattern: &RE) -> bool {
    let input_chars = input_line.chars().collect::<Vec<_>>();
    let mut idx = 0;
    let rtype_iter = re_pattern.rtype.iter().peekable();
    println!("[here] {:?}: {:?}", &input_line, re_pattern);
    for rtype in rtype_iter {
        if idx == input_chars.len() {
            return false;
        }
        println!("[here for] {:?}: {:?}", &input_line, rtype);
        match rtype {
            RType::Qplus(_) | RType::Qquestion(_) => {
                println!("[here for] calling +|? for rtype: {:?}", rtype);
                let tidx = match_quantifier(
                    &input_line[idx..],
                    &RE {
                        rtype: vec![rtype.clone()],
                        anchor: StringAnchor::None, // match_quantifier doesn't need to know about StringAnchor
                    },
                );
                println!("[here for] +|? tidx: {}", tidx);
                if let RType::Qplus(_) = rtype {
                    if tidx == 0 {
                        return false;
                    }
                }
                // TODO: what if idx is 0, and tidx is also 0
                // Fix this
                idx += tidx - 1;
            },
            RType::Ch(c) if &input_chars[idx] != c => {
                return false;
            },
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
                        return false;
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
                        return false;
                    }
                }
            },
            RType::Cgd => {
                if !input_chars[idx].is_ascii_digit() {
                    return false;
                }
            },
            RType::Cgw => {
                if !input_chars[idx].is_ascii_alphanumeric() {
                    return false;
                }
            },
            _ => {}
        }
        idx += 1;
    }
    if idx < input_chars.len() {
        if let StringAnchor::End = re_pattern.anchor {
            return false;
        }
    }
    true
}

pub fn match_pattern(input_line: &str, re: &RE) -> bool {
    if input_line.is_empty() {
        false
    } else if let StringAnchor::Start = re.anchor {
        match_here(input_line, &re)
    } else {
        if match_pattern(&input_line[1..], re) {
            return true;
        }
        if !match_here(input_line, &re) {
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
}
