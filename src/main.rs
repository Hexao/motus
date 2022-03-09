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
struct Args {
    pub search: Option<MaskDescriptor>,

    #[structopt(long)]
    pub bench: Option<char>,
}

// use https://www.tusmo.xyz/s3da53bb 4 tests
// inventees

fn main() {
    let start = std::time::Instant::now();

    let args = Args::from_args();
    let start = log_section(start, "Args parsed");

    if let Some(mask_desc) = args.search {
        let (dico, mut best) = match dico::load(mask_desc.dico, mask_desc.len + 1) {
            Ok(dico) => dico,
            Err(error) => {
                eprintln!("{}", error);
                return;
            }
        };

        if dico.is_empty() {
            print!("No world of len {} found in '{}.txt'", mask_desc.len, mask_desc.dico);
            return;
        }

        let mut mask = mask::Mask::new(mask_desc.dico, mask_desc.len);
        let mut result = mask::ResultState::new(mask_desc.len as usize + 1);
        log_section(start, "Dico loaded");

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

            let mut buf = String::with_capacity(10);

            match mask.filter(&dico) {
                Ok(possibility) => if possibility == 1 {
                    return;
                },
                Err(err) => {
                    eprintln!("{}", err);
                    return;
                }
            };

            result = loop {
                buf.clear();
                print!("Result: ");
                std::io::stdout().flush().unwrap();

                if let Err(err) = std::io::stdin().read_line(&mut buf) {
                    eprintln!("{}", err);
                    return;
                }

                match buf.trim().try_into() {
                    Ok(rs) => break rs,
                    Err(err) => eprintln!("{}", err),
                }
            };

            if let Err(err) = mask.update(&dico[word_id], &result) {
                eprintln!("{}", err);
                return;
            }

            // println!("{:?}", mask);
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
    }
}

fn log_section(start: std::time::Instant, msg: &str) -> std::time::Instant {
    println!("{} in {}µs", msg, start.elapsed().as_micros());
    std::time::Instant::now()
}
