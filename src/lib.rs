use std::str::Chars;

#[derive(Debug,Clone)]
pub enum RType {
    Ch(char),          // character
    Ccl(String, bool), // character group, +ve/-ve
    Cgd,               // character class digit
    Cgw,               // character class alphanumeric
    Qplus(Box<RType>), // match one ore more time for previous RType
}

// NOTE: we'll be ignoring multi-line regex, so start/end anchor for newline is ignored read:
// https://learn.microsoft.com/en-us/dotnet/standard/base-types/anchors-in-regular-expressions#start-of-string-only-a
pub enum StringAnchor {
    Start,
    End,
    None,
}

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
    let mut pidx: usize = 0;  // pattern index
    'out: loop {
        if chiter.peek().is_none() {
            break 'out RE {
                rtype: re_pattern,
                anchor: string_anchor,
            };
        }
        let c = chiter.next().unwrap();
        match c {
            '+' if pidx >= 1 => {
                re_pattern[pidx-1] = RType::Qplus(Box::new(re_pattern[pidx-1].clone()));
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

fn match_here(input_line: &str, re_pattern: &Vec<RType>) -> bool {
    let input_chars = input_line.chars().collect::<Vec<_>>();
    let mut idx = 0;
    for re in re_pattern.iter() {
        if idx == input_chars.len() {
            // NOTE: this will not be false if current RType is a quantifier
            return false;
        }
        match re {
            RType::Ch(c) if &input_chars[idx] != c => {
                return false;
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
            }
            RType::Cgd => {
                if !input_chars[idx].is_ascii_digit() {
                    return false;
                }
            }
            RType::Cgw => {
                if !input_chars[idx].is_ascii_alphanumeric() {
                    return false;
                }
            }
            _ => {}
        }
        idx += 1;
    }
    // if idx < input_chars.len() {
    //     // check for StringAnchor::End here
    // }
    true
}

pub fn match_pattern(input_line: &str, re: &RE) -> bool {
    if input_line.is_empty() {
        false
    } else if let StringAnchor::Start = re.anchor {
        match_here(input_line, &re.rtype)
    } else {
        if match_pattern(&input_line[1..], re) {
            return true;
        }
        if !match_here(input_line, &re.rtype) {
            return false;
        } /* else if let StringAnchor::End = re.anchor {
            // NOTE: will need to calculate the total len of chars matched based on pattern
            // beforehand for this to work, once we have to much for multiple occurences
            if input_line.len() > re.rtype.len() {
                return false;
            }
        } */
        true
    }
}
