use std::io::{ErrorKind, Read};

use structopt::StructOpt;

use crate::advents::AdventYear;

#[macro_use]
mod helper;

mod advent_2020;
mod advent_adapters;
mod advents;

#[derive(StructOpt, Debug)]
struct Cli {
    year: Option<u16>,
    advent: Option<u8>,
}

impl Cli {
    pub fn from_user(advent_years: &[AdventYear]) -> Self {
        let mut options: Self = Self::from_args();

        let dialoguer_theme = &dialoguer::theme::ColorfulTheme::default();

        if options.year.is_none() {
            let years: Vec<_> = advent_years.iter().map(|y| y.get_year()).collect();

            options.year = dialoguer::Select::with_theme(dialoguer_theme)
                .items(&years)
                .interact_opt()
                .unwrap()
                .map(|i| years[i]);
        }

        if let (Some(year), None) = (options.year, options.advent) {
            if let Some(advent_year) = advent_years.iter().find(|y| y.get_year() == year) {
                let advents: Vec<_> = advent_year
                    .iter()
                    .filter_map(|a| if a.skip() { None } else { Some(a.get_index()) })
                    .collect();

                options.advent = dialoguer::Select::with_theme(dialoguer_theme)
                    .items(&advents)
                    .interact_opt()
                    .unwrap()
                    .map(|i| advents[i]);
            };
        }

        options
    }
}

fn main() {
    let advent_years = vec![advent_2020::get_advent_year()];
    let options: Cli = Cli::from_user(&advent_years);

    match options.year {
        Some(year) => {
            match advent_years
                .into_iter()
                .find(|advent_year| advent_year.get_year() == year)
            {
                None => println!("No solution registered for given year {}", year),
                Some(target_year) => run_advent_year(&options, target_year),
            };
        }
        None => {
            advent_years
                .into_iter()
                .for_each(move |y| run_advent_year(&options, y));
        }
    }
}

fn run_advent_year(options: &Cli, y: advents::AdventYear) {
    let year = y.get_year();
    println!("Running year {}", year);

    let mut advents = y.into_advents();

    if advents.len() == 0 {
        return eprintln!("No adventures registered for year {}!", year);
    }

    advents.sort_by_key(|advent| advent.get_index());

    if let Some(advent) = options.advent {
        let index = advents
            .binary_search_by_key(&advent, |advent| advent.get_index())
            .expect("Advent index not found");
        let target_advent = advents.swap_remove(index);

        run_advent(year, target_advent);
    } else {
        advents
            .into_iter()
            .for_each(|advent| run_advent(year, advent));
    }
}

fn run_advent(year: u16, advent: Box<dyn advents::Advent>) {
    if advent.skip() {
        return println!("Skipping advent {}...", advent.get_index());
    }
    println!("Running advent day {}...", advent.get_index());

    let mut inputs = advent.get_input_names();
    let path_prefix = ["data", &year.to_string(), &advent.get_index().to_string()]
        .iter()
        .collect::<std::path::PathBuf>();

    std::fs::create_dir_all(&path_prefix).expect("could not create missing input data folder");

    for file in inputs.iter_mut() {
        let path = path_prefix.join(&file);
        file.clear();

        std::fs::File::open(&path)
            .and_then(|mut f| f.read_to_string(file))
            .and(Ok(()))
            .or_else(|err| {
                if err.kind() == ErrorKind::NotFound {
                    std::fs::File::create(&path).and(Ok(()))
                } else {
                    Err(err)
                }
            })
            .expect("could not read input file");
    }

    advent.process_input(inputs);

    println!("\n");
}
