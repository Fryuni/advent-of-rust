use std::collections::HashSet;

use crate::advent_adapters::AdventState;

#[derive(Clone)]
pub struct AdventDay17 {
    active_cells: HashSet<Coordinates>,
}

impl AdventDay17 {
    fn solve_step1(&mut self) -> usize {
        for _ in 0..6 {
            self.cycle_3d();
        }

        self.active_cells.len()
    }

    fn solve_step2(&mut self) -> usize {
        for _ in 0..6 {
            self.cycle_4d();
        }

        self.active_cells.len()
    }

    fn cycle_3d(&mut self) {
        self.active_cells = self
            .active_cells
            .iter()
            // Expand cloud of possibly affected cells
            .flat_map(|coordinate| coordinate.neighbors_3d())
            // Collect possibly affected cells in a set
            .collect::<HashSet<_>>()
            .into_iter()
            // Filter only cells that are active in the new generation
            .filter(|c| {
                let active_neighbors = c
                    .neighbors_3d()
                    .filter(|n| self.active_cells.contains(n))
                    .count();

                active_neighbors == 3 || (self.active_cells.contains(c) && active_neighbors == 2)
            })
            .collect();
    }

    fn cycle_4d(&mut self) {
        self.active_cells = self
            .active_cells
            .iter()
            // Expand cloud of possibly affected cells
            .flat_map(|coordinate| coordinate.neighbors_4d())
            // Collect possibly affected cells in a set
            .collect::<HashSet<_>>()
            .into_iter()
            // Filter only cells that are active in the new generation
            .filter(|c| {
                let active_neighbors = c
                    .neighbors_4d()
                    .filter(|n| self.active_cells.contains(n))
                    .count();

                active_neighbors == 3 || (self.active_cells.contains(c) && active_neighbors == 2)
            })
            .collect();
    }
}

impl AdventState for AdventDay17 {
    const INPUT_FILES: &'static [&'static str] = &["test.txt", "input.txt"];

    fn new(_input_file: &'static str, input_content: String) -> Self {
        Self {
            active_cells: input_content
                .split('\n')
                .enumerate()
                .flat_map(|(x, line)| {
                    line.chars()
                        .enumerate()
                        .filter(|&(_, char)| char == '#')
                        .map(move |(y, _)| Coordinates(x as isize, y as isize, 0, 0))
                })
                .collect(),
        }
    }

    fn run(self) {
        println!("Solution for step 1: {}", self.clone().solve_step1());
        println!("Solution for step 2: {}", self.clone().solve_step2());
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
struct Coordinates(isize, isize, isize, isize);

impl Coordinates {
    fn neighbors_3d(&self) -> impl Iterator<Item = Coordinates> {
        let &Coordinates(sx, sy, sz, sw) = self;
        (-1..=1)
            .flat_map(move |x| (-1..=1).flat_map(move |y| (-1..=1).map(move |z| (x, y, z))))
            .filter(|&(x, y, z)| !(x == y && y == z && z == 0))
            .map(move |(x, y, z)| Coordinates(sx + x, sy + y, sz + z, sw))
    }

    fn neighbors_4d(&self) -> impl Iterator<Item = Coordinates> {
        let &Coordinates(sx, sy, sz, sw) = self;

        (-1..=1)
            .flat_map(move |x| {
                (-1..=1).flat_map(move |y| {
                    (-1..=1).flat_map(move |z| (-1..=1).map(move |w| (x, y, z, w)))
                })
            })
            .filter(|&(x, y, z, w)| !(x == y && y == z && z == w && w == 0))
            .map(move |(x, y, z, w)| Coordinates(sx + x, sy + y, sz + z, sw + w))
    }
}

#[test]
fn test_coordinate_3d() {
    let coord = Coordinates(0, 0, 0, 0);

    assert_eq!(coord.neighbors_3d().count(), 26);

    for neighbor in coord.neighbors_3d() {
        neighbor
            .neighbors_3d()
            .position(|c| c == coord)
            .expect("neighbors must be reciprocated");
    }
}

#[test]
fn test_coordinate_4d() {
    let coord = Coordinates(0, 0, 0, 0);

    assert_eq!(coord.neighbors_4d().count(), 80);

    for neighbor in coord.neighbors_4d() {
        neighbor
            .neighbors_4d()
            .position(|c| c == coord)
            .expect("neighbors must be reciprocated");
    }
}
