use itertools::Itertools;
use nom::{
    bytes::complete::take,
    character::complete::{line_ending, u8},
    combinator::map,
    multi::{many1, separated_list1},
    Finish, Parser,
};
use std::{fs::read_to_string, usize};

#[derive(Debug, Clone)]
struct OctoSim {
    energy_map: Vec<Vec<u8>>,
    flash_victims: Vec<(usize, usize)>,
    flash_counter: u32,
}

impl OctoSim {
    fn new(energy_map: Vec<Vec<u8>>) -> Self {
        Self {
            energy_map,
            flash_victims: vec![],
            flash_counter: 0,
        }
    }
    fn width(&self) -> usize {
        self.energy_map[0].len()
    }
    fn height(&self) -> usize {
        self.energy_map.len()
    }
    fn step(&mut self) {
        let w = self.width();
        let h = self.height();
        let flash_list: Vec<_> = (0..w)
            .cartesian_product(0..h)
            .filter(|(x, y)| {
                self.energy_map[*x][*y] = (self.energy_map[*x][*y] + 1) % 10;
                self.energy_map[*x][*y] == 0
            })
            .collect();
        flash_list.iter().for_each(|pos| self.flash(*pos));
    }
    fn energy_up(&mut self, (x, y): (usize, usize)) {
        self.energy_map[x][y] = (self.energy_map[x][y] + 1) % 10;
        if self.energy_map[x][y] == 0 {
            self.flash((x, y));
        }
    }
    fn flash_others(&mut self) {
        while let Some((x, y)) = self.flash_victims.pop() {
            if self.energy_map[x][y] == 0 {
                continue;
            }
            self.energy_up((x, y));
        }
    }
    fn flash(&mut self, (x, y): (usize, usize)) {
        let width = self.width();
        let height = self.height();
        self.flash_counter += 1;
        self.flash_victims.extend(
            (-1..=1)
                .cartesian_product(-1..=1)
                .filter(|e| e != &(0, 0))
                .filter_map(|(dx, dy)| {
                    let x = x as isize;
                    let y = y as isize;
                    if x + dx >= 0
                        && y + dy >= 0
                        && x + dx < width as isize
                        && y + dy < height as isize
                    {
                        Some(((x + dx) as usize, (y + dy) as usize))
                    } else {
                        None
                    }
                }),
        );
        self.flash_others();
    }
}

fn parse(input: &str) -> anyhow::Result<OctoSim> {
    let energy = take(1usize).and_then(u8);
    let row = many1(energy);
    let energy_map = separated_list1(line_ending, row);
    let mut parser = map(energy_map, OctoSim::new);
    parser
        .parse(input)
        .finish()
        .map(|(_input, parsed)| parsed)
        .map_err(|_: nom::error::Error<&str>| anyhow::anyhow!("parser error"))
}
fn main() -> anyhow::Result<()> {
    let input = read_to_string("day-11/input.txt").unwrap();
    let mut parsed = parse(&input)?;
    let mut parsed2 = parsed.clone();
    let part1 = octo_flashes_count(&mut parsed);
    println!("part1 result is {}", part1);
    let part2 = all_flash_step(&mut parsed2);
    println!("part2 result is {}", part2);
    Ok(())
}

fn octo_flashes_count(sim: &mut OctoSim) -> u32 {
    for _ in 0..100 {
        sim.step();
    }
    sim.flash_counter
}
fn all_flash_step(sim: &mut OctoSim) -> u32 {
    for step in 0.. {
        sim.step();
        if sim.energy_map.iter().flatten().copied().all(|e| e == 0) {
            return step + 1;
        }
    }
    u32::MAX
}
#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"5483143223
2745854711
5264556173
6141336146
6357385478
4167524645
2176841721
6882881134
4846848554
5283751526"#;
    #[test]
    fn part1() -> anyhow::Result<()> {
        let mut parsed = parse(INPUT)?;
        assert_eq!(octo_flashes_count(&mut parsed), 1656);
        Ok(())
    }
    #[test]
    fn part2() -> anyhow::Result<()> {
        let mut parsed = parse(INPUT)?;
        assert_eq!(all_flash_step(&mut parsed), 195);
        Ok(())
    }
}
