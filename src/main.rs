use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

const EN_WORDS_FILE: &str = "./en_words.txt";
const EN_POSSIBILITIES_FILE: &str = "./en_possibilities.txt";
const PT_WORDS_FILE: &str = "./pt_words.txt";
const PT_POSSIBILITIES_FILE: &str = "./pt_possibilities.txt";
const BR_WORDS_FILE: &str = "./br_words.txt";
const BR_POSSIBILITIES_FILE: &str = "./br_possibilities.txt";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Restriction {
    Here(char, usize),
    NotHere(char, usize),
    NoMore(char),
}

impl Ord for Restriction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Here(_, self_idx), Self::Here(_, other_idx)) => self_idx.cmp(other_idx),
            (Self::Here(_, _), _) => std::cmp::Ordering::Less,
            (Self::NotHere(_, self_idx), Self::NotHere(_, other_idx)) => self_idx.cmp(other_idx),
            (Self::NotHere(_, _), Self::Here(_, _)) => std::cmp::Ordering::Greater,
            (Self::NotHere(_, _), Self::NoMore(_)) => std::cmp::Ordering::Less,
            (Self::NoMore(self_char), Self::NoMore(other_char)) => self_char.cmp(other_char),
            (Self::NoMore(_), _) => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for Restriction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

fn calculate_guesses(possibilities: &[String], words: &[String]) -> Vec<String> {
    // figure out which guess gives the most info
    let mut letter_counts: HashMap<char, usize> = HashMap::new();
    for possibility in possibilities.iter() {
        for letter in possibility.chars() {
            *letter_counts.entry(letter).or_default() += 1;
        }
    }

    let half = possibilities.len() / 2;
    let mut letter_counts_vec: VecDeque<_> = letter_counts
        .iter()
        .map(|(letter, count)| {
            (
                *letter,
                if half > *count {
                    half - count
                } else {
                    count - half
                },
            )
        })
        .collect();
    letter_counts_vec
        .make_contiguous()
        .sort_unstable_by_key(|tup| tup.1);

    let mut chosen = words.to_vec();
    let mut chosen_letters = 0;
    while !letter_counts_vec.is_empty() && chosen_letters < 5 {
        if chosen.len() == 1 {
            return chosen;
        }
        let (letter, _) = letter_counts_vec.pop_front().unwrap();
        let mut next_chosen = chosen.clone();
        next_chosen.retain(|word| word.contains(letter));
        if !next_chosen.is_empty() {
            chosen = next_chosen;
            chosen_letters += 1;
        }
    }

    chosen
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
    let mut restrictions: Vec<_> = guess_iter
        .zip(res_iter)
        .enumerate()
        .map(|(idx, (letter, num))| match num {
            2 => Restriction::Here(letter, idx),
            1 => Restriction::NotHere(letter, idx),
            0 => Restriction::NoMore(letter),
            _ => unreachable!(),
        })
        .collect();
    restrictions.sort_unstable();
    restrictions
}

fn update_possibilities(possibilities: &mut Vec<String>, restrictions: &[Restriction]) {
    // remove possibilities that don't respect any of the restrictions
    let respects = |possibility: &String| -> bool {
        let mut found = HashSet::new();
        for restriction in restrictions {
            match restriction {
                Restriction::Here(letter, idx) => {
                    if possibility.as_bytes()[*idx] != *letter as u8 {
                        return false;
                    }
                    found.insert(letter);
                }
                Restriction::NotHere(letter, idx) => {
                    let not_here = possibility.as_bytes()[*idx] != *letter as u8;
                    let else_where = possibility.as_bytes().contains(&(*letter as u8));
                    if !(not_here && else_where) {
                        return false;
                    }
                    found.insert(letter);
                }
                Restriction::NoMore(letter) => {
                    if possibility.contains(*letter) && !found.contains(letter) {
                        return false;
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

use clap::{ArgEnum, Parser};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum Language {
    En,
    Pt,
    Br,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(arg_enum)]
    language: Language,
}

fn load_words(file_name: &str) -> std::io::Result<Vec<String>> {
    Ok(BufReader::new(File::open(file_name)?)
        .lines()
        .map(Result::unwrap)
        .collect())
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let (possibilities_file, words_file) = {
        match args.language {
            Language::En => (EN_POSSIBILITIES_FILE, EN_WORDS_FILE),
            Language::Pt => (PT_POSSIBILITIES_FILE, PT_WORDS_FILE),
            Language::Br => (BR_POSSIBILITIES_FILE, BR_WORDS_FILE),
        }
    };

    let (mut possibilities, words) = (load_words(possibilities_file)?, load_words(words_file)?);
    loop {
        let guess_suggestions = calculate_guesses(&possibilities, &words);
        println!("I suggest you try one of the following:");
        for suggestion in guess_suggestions.iter().take(5) {
            println!("\t{suggestion}");
        }

        let new_restrictions = handle_guess();

        update_possibilities(&mut possibilities, &new_restrictions);

        if possibilities.len() <= 1 {
            break;
        }

        if possibilities.len() <= 5 {
            println!("I have narrowed it down to these ones if you're feeling lucky:");
            for word in &possibilities {
                println!("\t{word}");
            }
        } else {
            println!(
                "I have narrowed it down to {} possibilities",
                possibilities.len()
            );
        }
    }
    if let Some(answer) = possibilities.get(0) {
        println!("The answer should be: {}", answer);
    } else {
        println!("Answer not in my wordlist :(");
    }
    Ok(())
}
