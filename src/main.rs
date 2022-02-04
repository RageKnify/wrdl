use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

const ENGLISH_FILE: &'static str = "/usr/share/dict/words";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PositionRestriction {
    Here,
    NotHere,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExistRestriction {
    letter: char,
    idx: usize,
    restriction: PositionRestriction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Restriction {
    Exists(ExistRestriction),
    NotExists(char),
}

fn calculate_guess(possibilities: &[String]) -> String {
    // figure out which guess gives the most info
    let mut letter_counts: HashMap<char, usize> = HashMap::new();
    for possibility in possibilities.iter() {
        for letter in possibility.chars() {
            *letter_counts.entry(letter).or_default() += 1;
        }
    }
    let n = possibilities.len();
    let min_appearances = n / 2 - n / 3;
    let max_appearances = n / 2 + n / 3;

    let mut average_letters: Vec<_> = letter_counts
        .iter()
        .filter(|(_, &val)| val > min_appearances && val < max_appearances)
        .collect();

    average_letters.sort_unstable_by(|a, b| a.1.cmp(b.1));

    let mut chosen = possibilities.to_vec();
    while average_letters.len() > 0 {
        if chosen.len() == 1 {
            return chosen[0].clone();
        }
        let mid = average_letters.len() / 2;
        let (letter, _) = average_letters.remove(mid);
        let mut next_chosen = chosen.clone();
        next_chosen.retain(|possibility| possibility.contains(*letter));
        if next_chosen.len() > 0 {
            chosen = next_chosen;
        }
    }

    chosen[0].clone()
}

fn handle_guess() -> Vec<Restriction> {
    // ask user for used guess
    println!("What did you use for a guess?");
    print!("> ");
    use std::io::Write;
    std::io::stdout().flush().expect("flushing should work");
    let mut guess = String::new();
    let stdin = std::io::stdin(); // We get `Stdin` here.
    stdin
        .read_line(&mut guess)
        .expect("we should be able to read a line from stdin");
    let trimmed = guess.trim();
    if trimmed.len() != 5 {
        eprintln!("guess isn't of length 5, exiting");
        std::process::exit(-1);
    }

    // ask user for result with some digit encoding
    println!("What result did you get?");
    println!("Use 0 for wrong, 1 for wrong place and 2 for correct letter");
    print!("> ");
    std::io::stdout().flush().expect("flushing should work");
    let mut result = String::new();
    stdin
        .read_line(&mut result)
        .expect("we should be able to read a line from stdin");
    let trimmed_res = result.trim();

    if trimmed_res.len() != 5 {
        eprintln!("result isn't of length 5, exiting");
        std::process::exit(-1);
    }

    // convert to Restriction's
    let guess_iter = trimmed.chars();
    let res_iter = trimmed_res
        .chars()
        .map(|num| num.to_digit(10).expect("result should be digits"));
    guess_iter
        .zip(res_iter)
        .enumerate()
        .map(|(idx, (letter, num))| match num {
            2 => Restriction::Exists(ExistRestriction {
                letter,
                idx,
                restriction: PositionRestriction::Here,
            }),
            1 => Restriction::Exists(ExistRestriction {
                letter,
                idx,
                restriction: PositionRestriction::NotHere,
            }),
            0 => Restriction::NotExists(letter),
            _ => unreachable!(),
        })
        .collect()
}

fn update_possibilities(possibilities: &mut Vec<String>, restrictions: &[Restriction]) {
    // remove possibilities that don't respect any of the restrictions
    let respects = |possibility: &String| -> bool {
        for (possibility_idx, possibility_letter) in possibility.chars().enumerate() {
            for restriction in restrictions {
                match restriction {
                    Restriction::Exists(ExistRestriction {
                        letter,
                        idx,
                        restriction,
                    }) => {
                        match (
                            possibility_letter == *letter,
                            possibility_idx == *idx,
                            PositionRestriction::Here == *restriction,
                        ) {
                            (false, true, true) => {
                                return false;
                            }
                            (true, true, false) => {
                                return false;
                            }
                            _ => (),
                        }
                    }
                    Restriction::NotExists(bad_letter) => {
                        if possibility_letter == *bad_letter {
                            return false;
                        }
                    }
                }
            }
        }
        true
    };

    let mut i = 0;
    while i < possibilities.len() {
        if !respects(&possibilities[i]) {
            possibilities.remove(i);
        } else {
            i += 1;
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut possibilities: Vec<_> = BufReader::new(File::open(ENGLISH_FILE)?)
        .lines()
        .map(Result::unwrap)
        .filter(|l| l.len() == 5 && l.chars().all(|letter| letter.is_ascii_alphabetic()))
        .map(|l| l.to_lowercase())
        .collect();
    while possibilities.len() > 1 {
        let guess_suggestion = calculate_guess(&possibilities);
        println!("I think you should try: {}", guess_suggestion);

        let new_restrictions = handle_guess();

        update_possibilities(&mut possibilities, &new_restrictions);
    }
    if let Some(answer) = possibilities.get(1) {
        println!("The answer should be: {}", answer);
    } else {
        println!("Answer not in my wordlist :(");
    }
    Ok(())
}
