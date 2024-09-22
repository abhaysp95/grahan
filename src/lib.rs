#[derive(Debug, Clone, PartialEq)]
pub enum RType {
    Ch(char),                                // character
    Ccl(String, bool),                       // character group, +ve/-ve
    Cgd,                                     // character class digit
    Cgw,                                     // character class alphanumeric
    Qplus(Box<RType>),                       // match one ore more time for previous RType
    Qquestion(Box<RType>),                   // match zero or one time for previous RType
    Wildcard,                                // match any character
    AltOr(Box<Vec<RType>>, Box<Vec<RType>>), // match (a|b), a or b
    BackRefs(u8),                            // match for backref like \1
    Capture(Box<Vec<RType>>),                // capture multiple of RType within ()
}

// NOTE: we'll be ignoring multi-line regex, so start/end anchor for newline is ignored read:
// https://learn.microsoft.com/en-us/dotnet/standard/base-types/anchors-in-regular-expressions#start-of-string-only-a
#[derive(Debug, Clone, PartialEq)]
pub enum StringAnchor {
    Start,
    End,
    None,
}

#[derive(Debug, PartialEq)]
pub struct RE {
    pub rtype: Vec<RType>,
    pub anchor: StringAnchor,
    pub backrefs: Option<Vec<(Vec<RType>, Option<String>)>>,
}

pub fn get_regex_pattern(pattern: &str) -> RE {
    let mut re_pattern: Vec<RType> = vec![];
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
    let cpattern = pattern.chars().collect::<Vec<_>>();
    let mut pidx: usize = 0; // pattern index
    let mut idx = 0; // character index
    let mut backrefs = vec![];
    'out: loop {
        if idx == cpattern.len() {
            break 'out RE {
                rtype: re_pattern,
                anchor: string_anchor,
                backrefs: Some(backrefs),
            };
        }

        match cpattern[idx] {
            '+' | '?' if pidx >= 1 => {
                #[cfg(debug_assertions)]
                {
                    dbg!("match quantifier start");
                    dbg!(idx);
                    dbg!(pidx);
                    dbg!(re_pattern.len());
                    println!("{:?}", re_pattern);
                }
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
                #[cfg(debug_assertions)]
                dbg!("match quantifier end");
            }
            '\\' => {
                // TODO: add support to match '\' too, currently this logic ignores it
                while '\\' == cpattern[idx + 1] {
                    idx += 1;
                }
                idx += 1;
                re_pattern.push(match cpattern[idx] {
                    'd' => RType::Cgd,
                    'w' => RType::Cgw,
                    '+' => RType::Ch('+'),
                    '1'..'9' => {
                        // keeping it single digit for now
                        RType::BackRefs(
                            cpattern[idx]
                                .to_digit(10)
                                .expect("back-reference should be a number")
                                as u8,
                        )
                    }
                    _ => {
                        #[cfg(debug_assertions)]
                        println!("=> {}", cpattern[idx]);
                        unreachable!();
                    }
                });
            }
            '(' => {
                #[cfg(debug_assertions)]
                dbg!("match capture group start");
                let mut found = false;
                let mut closing_dist = 0;
                #[cfg(debug_assertions)]
                dbg!(idx);
                while idx + closing_dist < cpattern.len() {
                    if cpattern[idx + closing_dist] == ')' {
                        found = true;
                        break;
                    }
                    closing_dist += 1;
                }
                #[cfg(debug_assertions)]
                dbg!(closing_dist);
                if !found {
                    panic!("Invalid regex pattern provided. Missing closing ) for opened (")
                }
                let mut pipe_dist = 0;
                found = false;
                while idx + pipe_dist < cpattern.len() {
                    if cpattern[idx + pipe_dist] == '|' {
                        found = true;
                        break;
                    }
                    pipe_dist += 1;
                }
                #[cfg(debug_assertions)]
                dbg!(pipe_dist);
                if found {
                    let re_left =
                        get_regex_pattern(&cpattern[idx + 1..].iter().collect::<String>());
                    idx += pipe_dist;
                    #[cfg(debug_assertions)]
                    dbg!(idx);
                    let re_right =
                        get_regex_pattern(&cpattern[idx + 1..].iter().collect::<String>());
                    idx += closing_dist - pipe_dist;
                    #[cfg(debug_assertions)]
                    {
                        dbg!(idx);
                        dbg!(cpattern.len());
                    }
                    let rtype = RType::AltOr(Box::new(re_left.rtype), Box::new(re_right.rtype));
                    backrefs.push((vec![rtype.clone()], None));
                    re_pattern.push(rtype);
                } else {
                    let re = get_regex_pattern(&cpattern[idx + 1..].iter().collect::<String>());
                    idx += closing_dist;
                    #[cfg(debug_assertions)]
                    dbg!(idx);
                    // TODO: push capture group here
                    backrefs.push((re.rtype.clone(), None));
                    re_pattern.push(RType::Capture(Box::new(re.rtype)));
                }
                #[cfg(debug_assertions)]
                dbg!("match capture group end");
            }
            '|' | ')' => {
                // should go back to '(' match block
                return RE {
                    rtype: re_pattern,
                    anchor: StringAnchor::None,
                    backrefs: Some(backrefs),
                };
            }
            '[' => {
                // let's assume that we'll find ']' later on always, for now
                let mut gmode = true;
                if idx + 1 < cpattern.len() {
                    if cpattern[idx + 1] == '^' {
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
            }
            '.' => re_pattern.push(RType::Wildcard),
            _ => re_pattern.push(RType::Ch(cpattern[idx])),
        };
        pidx += 1;
        idx += 1;
        #[cfg(debug_assertions)]
        {
            println!("------------Reached End---------------");
            dbg!(idx);
            dbg!(pidx);
            println!("rp: {:?}", &re_pattern);
            println!("------------Ended---------------");
        }
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
            let mut new_re = RE {
                rtype: vec![*rtype.clone()],
                anchor: StringAnchor::None,
                backrefs: None,
            };
            #[cfg(debug_assertions)]
            println!("+|? rtype: {:?}, {:?}", re.rtype[0], &re);
            if let RType::Qplus(_) = re.rtype[0] {
                #[cfg(debug_assertions)]
                println!("+|? [if] new_re: {:?}", &new_re);
                while match_here(&input_line[idx..], &mut new_re).0 {
                    idx += 1;
                }
            } else if let RType::Qquestion(_) = re.rtype[0] {
                if match_here(&input_line[idx..], &mut new_re).0 {
                    idx += 1;
                }
            }
        }
        _ => {}
    }
    idx
}

fn match_here(input_line: &str, re_pattern: &mut RE) -> (bool, usize) {
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
                        backrefs: None,
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
            }
            RType::AltOr(re_left, re_right) => {
                let left_status = match_here(
                    &input_line[idx..],
                    &mut RE {
                        rtype: re_left.as_ref().clone(),
                        anchor: StringAnchor::None,
                        backrefs: None,
                    },
                );
                if !left_status.0 {
                    let right_status = match_here(
                        &input_line[idx..],
                        &mut RE {
                            rtype: re_right.as_ref().clone(),
                            anchor: StringAnchor::None,
                            backrefs: None,
                        },
                    );
                    if !right_status.0 {
                        return (false, idx + left_status.1);
                    }
                    if let Some(ref mut backrefs) = re_pattern.backrefs {
                        for backref in backrefs.iter_mut() {
                            match backref.1 {
                                Some(_) => {}
                                None => {
                                    backref.1 =
                                        Some(input_line[idx..idx + right_status.1].to_string());
                                    break;
                                }
                            }
                        }
                        #[cfg(debug_assertions)]
                        println!("new backrefs: {:?}", re_pattern.backrefs.as_ref().unwrap());
                    }
                    if right_status.1 > 0 {
                        idx += right_status.1 - 1;
                    }
                } else {
                    if let Some(ref mut backrefs) = re_pattern.backrefs {
                        for backref in backrefs.iter_mut() {
                            match backref.1 {
                                Some(_) => {}
                                None => {
                                    backref.1 =
                                        Some(input_line[idx..idx + left_status.1].to_string());
                                    break;
                                }
                            }
                        }
                        #[cfg(debug_assertions)]
                        println!("new backrefs: {:?}", re_pattern.backrefs.as_ref().unwrap());
                    }
                    if left_status.1 > 0 {
                        idx += left_status.1 - 1;
                    }
                }
            }
            RType::Capture(cg) => {
                #[cfg(debug_assertions)]
                println!(
                    "Capture idx: {}, cg: {:?}, input_line: {}",
                    idx,
                    &cg,
                    &input_line[idx..]
                );
                let match_status = match_here(
                    &input_line[idx..],
                    &mut RE {
                        rtype: cg.as_ref().clone(),
                        anchor: StringAnchor::None,
                        backrefs: None,
                    },
                );
                if !match_status.0 {
                    return (false, idx + match_status.1);
                }
                #[cfg(debug_assertions)]
                println!("Captured: {:?}", &input_line[idx..idx + match_status.1]);
                if let Some(ref mut backrefs) = re_pattern.backrefs {
                    for backref in backrefs.iter_mut() {
                        match backref.1 {
                            Some(_) => {}
                            None => {
                                backref.1 = Some(input_line[idx..idx + match_status.1].to_string());
                                break;
                            }
                        }
                    }
                    #[cfg(debug_assertions)]
                    println!("new backrefs: {:?}", re_pattern.backrefs.as_ref().unwrap());
                }
                if match_status.1 > 0 {
                    idx += match_status.1 - 1;
                }
            }
            RType::BackRefs(bnum) => {
                if *bnum == 0 {
                    panic!("Backref can't be 0");
                }
                if let Some(backrefs) = re_pattern.backrefs.as_ref() {
                    if *bnum - 1 >= backrefs.len() as u8 {
                        panic!("Not enough capture groups for backref count: {}", *bnum)
                    }
                    #[cfg(debug_assertions)]
                    println!(
                        "Backrefs idx: {}, bnum: {:?}, backref: {:?}, input_line: {}",
                        idx,
                        bnum,
                        &backrefs,
                        &input_line[idx..]
                    );
                    let backref = &backrefs[(*bnum - 1) as usize];
                    if let Some(captured_string) = backref.1.as_ref() {
                        if idx + captured_string.len() > input_line.len() {
                            panic!(
                                "String length not enough to match capture group.
                                idx: {}, captured_string.len(): {}, input_line.len(): {}, bnum: {}",
                                idx,
                                captured_string.len(),
                                input_line.len(),
                                *bnum
                            );
                        }
                        if !captured_string.eq(&input_line[idx..idx + captured_string.len()]) {
                            #[cfg(debug_assertions)]
                            println!(
                                "Failed match, captured_string: {}, actual_string: {}",
                                captured_string,
                                &input_line[idx..idx + captured_string.len()]
                            );
                            return (false, idx + captured_string.len());
                        }
                        if captured_string.len() > 0 {
                            idx += captured_string.len() - 1;
                        }
                    }
                }
            }
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

pub fn match_pattern(input_line: &str, mut re: &mut RE) -> bool {
    if input_line.is_empty() {
        false
    } else if let StringAnchor::Start = re.anchor {
        match_here(input_line, &mut re).0
    } else {
        if match_pattern(&input_line[1..], re) {
            return true;
        }
        if !match_here(input_line, &mut re).0 {
            if let Some(backrefs) = re.backrefs.as_mut() {
                for backref in backrefs.iter_mut() {
                    backref.1 = None;
                }
            }
            return false;
        }
        true
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn quantifier_plus() {
        let re = get_regex_pattern("o+");
        assert_eq!(match_quantifier("oo", &re), 2);
    }

    #[test]
    fn full_pattern_quantifier_plus() {
        let mut re_pattern = get_regex_pattern("g+o+$");
        let input_line = "logs are good";
        assert_eq!(match_pattern(&input_line, &mut re_pattern), false);
        let mut re_pattern = get_regex_pattern("g+o+d$");
        assert_eq!(match_pattern(&input_line, &mut re_pattern), true);
    }

    #[test]
    fn full_pattern_quantifier_question() {
        let mut re_pattern = get_regex_pattern("g+l?o+d$");
        let input_line = "logs are good";
        assert_eq!(match_pattern(&input_line, &mut re_pattern), true);
        let mut re_pattern = get_regex_pattern("ca?t");
        let input_line = "cat";
        assert_eq!(match_pattern(&input_line, &mut re_pattern), true);
    }

    #[test]
    fn regex_pattern_backref_test() {
        let re_string = "e.(g+|h?)o+\\1d$";
        let expected_re = RE {
            rtype: vec![
                RType::Ch('e'),
                RType::Wildcard,
                RType::AltOr(
                    Box::new(vec![RType::Qplus(Box::new(RType::Ch('g')))]),
                    Box::new(vec![RType::Qquestion(Box::new(RType::Ch('h')))]),
                ),
                RType::Qplus(Box::new(RType::Ch('o'))),
                RType::BackRefs(1),
                RType::Ch('d'),
            ],
            anchor: crate::StringAnchor::End,
            backrefs: Some(vec![(
                vec![RType::AltOr(
                    Box::new(vec![RType::Qplus(Box::new(RType::Ch('g')))]),
                    Box::new(vec![RType::Qquestion(Box::new(RType::Ch('h')))]),
                )],
                None,
            )]),
        };
        let actual_re = get_regex_pattern(re_string);
        assert_eq!(actual_re, expected_re);
    }

    #[test]
    fn regex_pattern_multi_backref_test() {
        let re_string = "e.(g+|h?)(ld)o+\\1d\\2$";
        let expected_re = RE {
            rtype: vec![
                RType::Ch('e'),
                RType::Wildcard,
                RType::AltOr(
                    Box::new(vec![RType::Qplus(Box::new(RType::Ch('g')))]),
                    Box::new(vec![RType::Qquestion(Box::new(RType::Ch('h')))]),
                ),
                RType::Capture(Box::new(vec![RType::Ch('l'), RType::Ch('d')])),
                RType::Qplus(Box::new(RType::Ch('o'))),
                RType::BackRefs(1),
                RType::Ch('d'),
                RType::BackRefs(2),
            ],
            anchor: crate::StringAnchor::End,
            backrefs: Some(vec![
                (
                    vec![RType::AltOr(
                        Box::new(vec![RType::Qplus(Box::new(RType::Ch('g')))]),
                        Box::new(vec![RType::Qquestion(Box::new(RType::Ch('h')))]),
                    )],
                    None,
                ),
                (vec![RType::Ch('l'), RType::Ch('d')], None),
            ]),
        };
        let actual_re = get_regex_pattern(re_string);
        assert_eq!(actual_re, expected_re);
    }
}
