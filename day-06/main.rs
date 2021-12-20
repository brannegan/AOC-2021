use nom::{
    character::complete::{char, u8},
    combinator::map,
    multi::separated_list1,
    Finish, Parser,
};
use std::fs::read_to_string;

#[derive(Debug, Clone)]
struct FishSim {
    fishes: [u64; FishSim::FIRST_CYCLE + 1],
}
impl FishSim {
    const FIRST_CYCLE: usize = 8;
    const NORMAL_CYCLE: usize = 6;
    fn new(initial: Vec<u8>) -> Self {
        let mut fishes = [0; FishSim::FIRST_CYCLE + 1];

        for fish in initial {
            fishes[fish as usize] += 1;
        }
        Self { fishes }
    }

    fn fishes_after(&mut self, days: u32) -> u64 {
        for _ in 1..=days {
            self.fishes.rotate_left(1);
            self.fishes[FishSim::NORMAL_CYCLE] += self.fishes[FishSim::FIRST_CYCLE];
        }
        self.fishes.into_iter().sum::<u64>()
    }
}

fn parse(input: &str) -> anyhow::Result<FishSim> {
    let fishes = separated_list1(char(','), u8);
    let mut parser = map(fishes, FishSim::new);
    parser
        .parse(input)
        .finish()
        .map(|(_input, parsed)| parsed)
        .map_err(|_: nom::error::Error<&str>| anyhow::anyhow!("parser error"))
}
fn main() -> anyhow::Result<()> {
    let input = read_to_string("day-06/input.txt").unwrap();
    let mut sim = parse(&input)?;
    let part1 = sim.fishes_after(80);
    println!("part1 result is {}", part1);
    let part2 = sim.fishes_after(256 - 80);
    println!("part2 result is {}", part2);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const INPUT: &str = "3,4,3,1,2";
    #[test]
    fn part1() -> anyhow::Result<()> {
        let mut sim = parse(INPUT)?;
        assert_eq!(sim.fishes_after(18), 26);
        assert_eq!(sim.fishes_after(80 - 18), 5934);
        Ok(())
    }
    #[test]
    fn part2() -> anyhow::Result<()> {
        let mut sim = parse(INPUT)?;
        assert_eq!(sim.fishes_after(256), 26984457539);
        Ok(())
    }
}
