use std::env;
use std::io;
use std::process;

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
    } else if pattern.starts_with('[') && pattern.ends_with(']') {
        for p in pattern[1..pattern.len()-1].chars() {
            if input_line.contains(p) {
                return true;
            }
        }
        false
    }
    else if pattern.chars().count() == 1 {
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

    if match_pattern(&input_line, &pattern) {
        process::exit(0)
    } else {
        process::exit(1)
    }
}
