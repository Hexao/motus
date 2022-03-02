use structopt::StructOpt;
mod word;
mod dico;

#[derive(StructOpt)]
struct Args {
    pub mask: String,

    pub exclude: Option<String>,
    pub include: Option<String>,

    #[structopt(long)]
    pub add: bool,
}

// use https://www.tusmo.xyz/s3da53bb 4 tests

fn main() {
    let start = std::time::Instant::now();
    let args = Args::from_args();

    if args.mask.len() < 2 {
        eprintln!("Mask must have at least two characters");
        return;
    }

    if args.add {
        dico::add(&args.mask);
        return;
    }

    let dico = match dico::load(&args.mask) {
        Ok(dico) => dico,
        Err(error) => {
            eprintln!("{}", error);
            return;
        }
    };

    if dico.len() == 0 {
        print!("No world found for mask '{}'", args.mask);
        return;
    }

    let dico = if let Some(exclude) = args.exclude {
        let mut dico_exclude = Vec::with_capacity(dico.len());

        for word in dico {
            let pass_filter = word.chars().all(|c| {
                exclude.chars().all(|ex| c != ex)
            });

            if pass_filter {
                dico_exclude.push(word);
            }
        }

        if dico_exclude.len() == 0 {
            print!("No world found after exclude '{}'", exclude);
            return;
        }

        if let Some(include) = args.include {
            let mut dico_include = Vec::with_capacity(dico_exclude.len());

            for word in dico_exclude {
                let pass_filter = include.chars().all(|inc| {
                    word.chars().any(|c| c == inc)
                });

                if pass_filter {
                    dico_include.push(word);
                }
            }

            if dico_include.len() == 0 {
                print!("No world found after include '{}'", include);
                return;
            }

            dico_include
        } else {
            dico_exclude
        }
    } else {
        dico
    };

    println!("Search in {}µs", start.elapsed().as_micros());
    println!("{:?}", dico);
}
