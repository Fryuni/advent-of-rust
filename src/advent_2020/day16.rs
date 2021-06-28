use std::iter::FromIterator;
use std::str::FromStr;

use crate::advents::Advent;

pub struct AdventDay16;

#[derive(Debug)]
enum ValidValue {
    Single(usize),
    Range(usize, usize),
}

#[derive(Debug)]
struct PossibleField {
    name: String,
    valid_values: Vec<ValidValue>,
}

impl PossibleField {
    fn fits(&self, value: usize) -> bool {
        self.valid_values.iter().any(|val| match val {
            ValidValue::Single(v) => v == &value,
            ValidValue::Range(min, max) => min <= &value && max >= &value,
        })
    }
}

struct PossibleFields(Vec<PossibleField>);

impl PossibleFields {
    fn fits(&self, value: usize) -> bool {
        self.0.iter().any(|v| v.fits(value))
    }
}

impl AdventDay16 {
    fn process_sample(&self, data: String) {
        let mut lines = data.split('\n');

        // Parse possible fields
        let fields: PossibleFields = lines
            .by_ref()
            .take_while(|l| !l.is_empty())
            .map(FromStr::from_str)
            .collect::<Result<_, _>>()
            .expect("could not parse possible fields");

        // Sanity check
        assert_eq!(lines.next().unwrap(), "your ticket:");

        let my_ticket: Vec<usize> = lines
            .next()
            .expect("missing 'my ticket' line")
            .split(',')
            .map(FromStr::from_str)
            .collect::<Result<_, _>>()
            .expect("invalid 'my ticket' line");

        // Sanity check
        assert_eq!(lines.next().unwrap(), "");
        assert_eq!(lines.next().unwrap(), "nearby tickets:");

        let mut nearby_tickets: Vec<Vec<usize>> = lines
            .take_while(|s| !s.is_empty())
            .map(|line| {
                line.split(',')
                    .map(FromStr::from_str)
                    .collect::<Result<_, _>>()
            })
            .collect::<Result<_, _>>()
            .expect("could not parse nearby tickets");

        // Step 1: Calculate the scanning error rate
        let ticket_scanning_error_rate: usize = nearby_tickets
            .iter()
            .flat_map(|v| v.iter())
            .filter(|&&v| !fields.fits(v))
            .copied()
            .sum();

        println!("Answer to step 1 is: {}", ticket_scanning_error_rate);

        // Discard all invalid tickets
        for i in (0..nearby_tickets.len()).rev() {
            if nearby_tickets[i].iter().any(|&f| !fields.fits(f)) {
                nearby_tickets.swap_remove(i);
            }
        }
        println!("{} valid tickets", nearby_tickets.len());

        let field_solution = self.solve_fields(&fields, &nearby_tickets);

        let solution: usize = my_ticket
            .into_iter()
            .zip(field_solution.into_iter())
            .filter_map(|(field_val, name)| {
                if name.starts_with("departure") {
                    Some(field_val)
                } else {
                    None
                }
            })
            .product();

        println!("Answer to step 2 is: {}", solution);
    }

    fn solve_fields<'a>(
        &self,
        fields: &'a PossibleFields,
        nearby_fields: &[Vec<usize>],
    ) -> Vec<&'a str> {
        let mut field_possibilities: Vec<Vec<&PossibleField>> =
            vec![fields.0.iter().collect(); nearby_fields[0].len()];

        let mut field_solution: Vec<Option<&str>> = vec![None; field_possibilities.len()];

        for ticket in nearby_fields {
            for (field_idx, &value) in ticket.iter().enumerate() {
                let field_desc = &mut field_possibilities[field_idx];

                let exclusion: Vec<_> = field_desc
                    .iter()
                    .enumerate()
                    .rev()
                    .filter(|(_, &field)| !field.fits(value))
                    .map(|(idx, _)| idx)
                    .collect();

                for idx in exclusion {
                    field_desc.swap_remove(idx);
                }
            }
        }

        'outer: loop {
            for field_idx in 0..field_possibilities.len() {
                if field_possibilities[field_idx].len() == 1 {
                    let field = field_possibilities[field_idx].pop().unwrap();
                    for desc in field_possibilities.iter_mut() {
                        if let Some(remove_idx) = desc.iter().position(|&f| f.name == field.name) {
                            desc.swap_remove(remove_idx);
                        }
                    }

                    field_solution[field_idx] = Some(&field.name);

                    continue 'outer;
                }
            }
            break;
        }

        field_solution
            .into_iter()
            .collect::<Option<_>>()
            .expect("could not solve all fields")
    }
}

impl Advent for AdventDay16 {
    fn get_index(&self) -> u8 {
        16
    }

    fn get_input_names(&self) -> Vec<String> {
        vec![
            "test.txt".to_owned(),
            "test2.txt".to_owned(),
            "input.txt".to_owned(),
        ]
    }

    fn process_input(&self, data: Vec<String>) {
        data.into_iter()
            .zip(["test", "test2", "real"])
            .for_each(|(d, name)| {
                println!();
                println!("Processing '{}' input", name);
                self.process_sample(d);
            })
    }
}

impl FromStr for ValidValue {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((left, right)) = s.split_once('-') {
            Ok(ValidValue::Range(
                left.parse().or(Err("could not parse valid value"))?,
                right.parse().or(Err("could not parse valid value"))?,
            ))
        } else {
            Ok(ValidValue::Single(
                s.parse().or(Err("could not parse valid value"))?,
            ))
        }
    }
}

impl FromStr for PossibleField {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, options) = s.split_once(": ").ok_or("malformed header line")?;

        Ok(Self {
            name: name.to_owned(),
            valid_values: options
                .split(" or ")
                .map(|v| v.parse())
                .collect::<Result<_, _>>()?,
        })
    }
}

impl FromIterator<PossibleField> for PossibleFields {
    fn from_iter<T: IntoIterator<Item = PossibleField>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
