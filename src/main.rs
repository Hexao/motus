use std::io::Write;

use structopt::StructOpt;
mod mask;
mod dico;

struct MaskDescriptor {
    pub dico: char,
    pub len: u8,
}

impl std::str::FromStr for MaskDescriptor {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            Err("MaskDescriptor must be of len 2!")
        } else {
            let mut iter = s.chars();
            let dico = iter.next().unwrap();

            let len = match iter.next().unwrap() {
                len @ '5'..='8' => Ok(len as u8 - b'0'),
                _ => Err("len must be between 5 and 9 included"),
            }?;

            Ok(Self { dico, len })
        }
    }
}

#[derive(StructOpt)]
/// is some flag is provided, search pattern will be ignored
struct Args {
    /// shound match pattern /[a-z][5-8]/
    pub search: Option<MaskDescriptor>,

    #[structopt(long, short)]
    /// You want play but you haven't friends ? So just give it a word
    pub auto: Option<String>,

    #[structopt(long, short)]
    /// bench a specific dictionary. <bench> must be letter
    pub bench: Option<char>,
}

// use https://www.tusmo.xyz/s3da53bb 4 tests
// inventees

fn main() {
    let mut args = Args::from_args();

    if let Some(word) = &args.auto {
        let word_len = word.len();

        if !(6..=9).contains(&word_len) {
            eprintln!("word don't have the correct length!");
            args.search = None;
            args.bench = None;
        } else {
            let dico = word.chars().next().unwrap();
            args.search = Some(MaskDescriptor { dico, len: word_len as u8 - 1 });
        }
    }

    if let Some(mask_desc) = args.search {
        let start = std::time::Instant::now();
        let (dico, mut best) = match dico::load(mask_desc.dico, mask_desc.len + 1) {
            Ok(dico) => dico,
            Err(error) => {
                eprintln!("{}", error);
                return;
            }
        };

        if dico.is_empty() {
            eprintln!("No world of len {} found in '{}.txt'", mask_desc.len, mask_desc.dico);
            return;
        }

        if let Some(word) = &args.auto {
            if !dico.contains(word) {
                eprintln!("The word '{}' won't be found: not in the dictionary", word);
                return;
            }
        }

        let mut mask = mask::Mask::new(mask_desc.dico, mask_desc.len);
        let mut result = mask::ResultState::new(mask_desc.len as usize + 1);
        println!("Dico loaded in {}µs ({} words)", start.elapsed().as_micros(), dico.len());

        while !result.complet() {
            let start = std::time::Instant::now();

            let word_id = if best.is_some() {
                let word_id = best.unwrap(); // SAFE
                best = None;

                println!("Best word: {}", dico[word_id]);
                word_id
            } else {
                let (word_id, score) = match mask.find_best(&dico) {
                    Ok(stats) => stats,
                    Err(err) => {
                        eprintln!("{}", err);
                        return;
                    }
                };

                println!("Word found in {:.2}s", start.elapsed().as_secs_f32());
                println!("Best word: {} ({:.2})", dico[word_id], score);
                word_id
            };

            if let Some(word) = &args.auto {
                if let Err(err) = result.update_with(&dico[word_id], word) {
                    eprintln!("{}", err);
                    return;
                }

                println!("Result: {}", result);

                if let Err(err) = mask.update(&dico[word_id], &result) {
                    eprintln!("{}", err);
                    return;
                }
            } else {
                let mut buf = String::with_capacity(10);

                result = loop {
                    buf.clear();
                    print!("Result: ");
                    std::io::stdout().flush().unwrap();

                    if let Err(err) = std::io::stdin().read_line(&mut buf) {
                        eprintln!("{}", err);
                        return;
                    }

                    let rs: mask::ResultState = match buf.trim().try_into() {
                        Ok(rs) => rs,
                        Err(err) => {
                            eprintln!("{}", err);
                            continue;
                        }
                    };

                    match mask.update(&dico[word_id], &rs) {
                        Err(err) => eprintln!("{}", err),
                        Ok(()) => break rs,
                    }
                }
            };

            // println!("{:?}", mask);

            match mask.filter(&dico) {
                Ok(possibilities) => {
                    if possibilities == 1 {
                        if !result.complet() {
                            let word_id = match mask.find_best(&dico) {
                                Ok((word_id, _)) => word_id,
                                Err(err) => {
                                    eprintln!("{}", err);
                                    return;
                                }
                            };

                            println!("Obviously: {}", dico[word_id]);
                            return;
                        }
                    } else {
                        println!("{} words remaining", possibilities);
                    }
                }
                Err(err) => {
                    eprintln!("{}", err);
                    return;
                }
            }
        }
    } else if let Some(char) = args.bench {
        for word_len in 6..=9 {
            let start = std::time::Instant::now();
            let dico = match dico::load(char, word_len) {
                Ok((dico, _)) => dico,
                Err(error) => {
                    eprintln!("{}", error);
                    return;
                }
            };

            let mask = mask::Mask::new(char, word_len - 1);
            let (word_id, _) = match mask.find_best(&dico) {
                Ok(id) => id,
                Err(err) => {
                    eprintln!("{}", err);
                    return;
                }
            };

            println!("For words of len {}, best word is {} ({}) in {:.2}s", word_len, dico[word_id], word_id, start.elapsed().as_secs_f32());
        }
    } else if let Err(err) = Args::clap().print_help() {
        eprintln!("{}", err);
    }
}
