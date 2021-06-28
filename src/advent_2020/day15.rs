use crate::advents::Advent;
use std::collections::HashMap;

pub struct AdventDay15;

impl Advent for AdventDay15 {
    fn get_index(&self) -> u8 {
        15
    }

    fn process_input(&self, mut data: Vec<String>) {
        let mut seq_state: Vec<usize> = data
            .pop()
            .unwrap()
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect();

        let mut next_value = seq_state.pop().unwrap();
        let mut current_turn = seq_state.len() + 1;

        let mut entries: HashMap<usize, usize> = seq_state
            .iter()
            .cloned()
            .enumerate()
            .map(|(v, k)| (k, v + 1))
            .collect();

        while current_turn < 30_000_000 {
            if current_turn == 2020 {
                println!("The response for stage 1 is: {}", next_value);
            }

            let last_occurrence = entries.entry(next_value).or_insert(current_turn);

            next_value = current_turn - *last_occurrence;
            *last_occurrence = current_turn;

            current_turn += 1;
        }

        println!("The response for stage 2 is: {}", next_value);
    }
}
