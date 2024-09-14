use std::env;
use std::io;
use std::process;

#[derive(Debug)]
enum RType {
    Ch(char),          // character
    Ccl(String, bool), // character group, +ve/-ve
    Cgd,               // character class digit
    Cgw,               // character class alphanumeric
}

fn get_regex_pattern(pattern: &str) -> Vec<RType> {
    let mut re_pattern: Vec<RType> = vec![];
    let mut chiter = pattern.chars().peekable();
    'out: loop {
        if chiter.peek().is_none() {
            break 'out re_pattern;
        }
        let c = chiter.next().unwrap();
        re_pattern.push(match c {
            '\\' => match chiter.next().unwrap() {
                'd' => RType::Cgd,
                'w' => RType::Cgw,
                _ => unreachable!(),
            },
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

// fn match_here(idx: usize, input_line: &str, re_pattern: &Vec<RType>) -> bool {
//
// }

fn match_pattern_remastered(input_line: &str, re_pattern: &Vec<RType>) -> bool {
    if input_line.is_empty() {
        false
    } else {
        if match_pattern_remastered(&input_line[1..], re_pattern) {
            println!("= {}", &input_line);
            return true;
        }
        let input_chars = input_line.chars().collect::<Vec<_>>();
        let mut idx = 0;
        for re in re_pattern.iter() {
            match re {
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
                    }
                }
                _ => {},
            }
            idx += 1;
        }
        if input_line.len() == 3 {
            true
        } else {
            false
        }
    }
}

fn match_pattern(input_line: &str, pattern: &str) -> bool {
    if pattern == "\\d" {
        for c in input_line.chars() {
            if c.is_ascii_digit() {
                return true;
            }
        }
        false
    } else if pattern == "\\w" {
        for c in input_line.chars() {
            if !c.is_ascii_alphanumeric() && c != '_' {
                return false;
            }
        }
        true
    } else if pattern.starts_with("[^") && pattern.ends_with(']') {
        let pattern = &pattern[2..pattern.len() - 1];
        for pc in pattern.chars() {
            if input_line.contains(pc) {
                return false;
            }
        }
        true
    } else if pattern.starts_with('[') && pattern.ends_with(']') {
        for pc in pattern[1..pattern.len() - 1].chars() {
            if input_line.contains(pc) {
                return true;
            }
        }
        false
    } else if pattern.chars().count() == 1 {
        return input_line.contains(pattern);
    } else {
        panic!("Unhandled pattern: {}", pattern)
    }
}

// Usage: echo <input_text> | your_program.sh -E <pattern>
fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    let re_pattern = get_regex_pattern(&pattern);
    for re in re_pattern.iter() {
        println!("{:?}", re)
    }

    if match_pattern_remastered(&input_line, &re_pattern) {
        process::exit(0);
    } else {
        process::exit(1);
    }

    // if match_pattern(&input_line, &pattern) {
    //     process::exit(0)
    // } else {
    //     process::exit(1)
    // }
}
