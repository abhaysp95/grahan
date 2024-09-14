use std::env;
use std::io;
use std::process;

#[derive(Debug)]
enum RType {
    ch(char),          // character
    ccl(String, bool), // character group, +ve/-ve
    cgd,               // character class digit
    cgw,               // character class alphanumeric
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
                'd' => RType::cgd,
                'w' => RType::cgw,
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
                RType::ccl(group, gmode)
            }
            _ => RType::ch(c),
        });
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
    for re in re_pattern {
        println!("{:?}", re)
    }

    // if match_pattern(&input_line, &pattern) {
    //     process::exit(0)
    // } else {
    //     process::exit(1)
    // }
}
