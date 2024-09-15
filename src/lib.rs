use std::str::Chars;

#[derive(Debug)]
pub enum RType {
    Ch(char),          // character
    Ccl(String, bool), // character group, +ve/-ve
    Cgd,               // character class digit
    Cgw,               // character class alphanumeric
}

// NOTE: we'll be ignoring multi-line regex, so start/end anchor for newline is ignored read:
// https://learn.microsoft.com/en-us/dotnet/standard/base-types/anchors-in-regular-expressions#start-of-string-only-a
pub enum StringAnchor {
    Start, End, None
}

pub struct RE {
    rtype: Vec<RType>,
    anchor: StringAnchor,
}

pub fn get_regex_pattern(pattern: &str) -> RE {
    let mut re_pattern: Vec<RType> = vec![];
    let mut chiter: std::iter::Peekable<Chars>;
    let mut string_anchor = StringAnchor::None;
    if pattern.starts_with('^') {
        string_anchor = StringAnchor::Start;
        chiter = pattern[1..].chars().peekable();
    } else {
        chiter = pattern.chars().peekable();
    }
    'out: loop {
        if chiter.peek().is_none() {
            break 'out RE {
                rtype: re_pattern,
                anchor: string_anchor,
            }
        }
        let c = chiter.next().unwrap();
        re_pattern.push(match c {
            '\\' => {
                while let Some('\\') = chiter.peek() {
                    chiter.next();
                }
                match chiter.next().unwrap() {
                    'd' => RType::Cgd,
                    'w' => RType::Cgw,
                    _ => {
                        println!("=> {}", c);
                        unreachable!();
                    }
                }
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
                RType::Ccl(group, gmode)
            }
            _ => RType::Ch(c),
        });
    }
}

fn match_here(input_line: &str, re_pattern: &Vec<RType>) -> bool {
        let input_chars = input_line.chars().collect::<Vec<_>>();
        let mut idx = 0;
        for re in re_pattern.iter() {
            if idx == input_chars.len() {
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
        }
        true
    }
}
