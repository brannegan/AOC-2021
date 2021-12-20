use nom::{
    character::complete::{char, i32},
    multi::separated_list1,
    Finish, Parser,
};
use std::fs::read_to_string;

fn parse(input: &str) -> anyhow::Result<Vec<i32>> {
    separated_list1(char(','), i32)
        .parse(input)
        .finish()
        .map(|(_input, parsed)| parsed)
        .map_err(|_: nom::error::Error<&str>| anyhow::anyhow!("parser error"))
}
fn alignment_cost_constant(offsets: &[i32]) -> i32 {
    let mut local = offsets.to_vec();
    local.copy_from_slice(offsets);
    local.sort_unstable();
    // SAFETY: obviously in bounds [0..len / 2..len]
    let median = unsafe { local.get_unchecked(local.len() / 2) };
    local.iter().fold(0, |acc, o| acc + (median - o).abs())
}
fn alignment_cost_progressive(offsets: &[i32]) -> i32 {
    let mut local = offsets.to_vec();
    local.copy_from_slice(offsets);
    let sum = local.iter().copied().sum::<i32>();
    let len = local.len() as i32;
    let mean_floor = sum / len as i32;
    let mean_ceil = mean_floor + 1;
    let fuel_for_mean = local.iter().fold(0, |acc, o| {
        acc + ((o - mean_floor).abs() * ((o - mean_floor).abs() + 1) / 2) //Fold on arithmetic progression
    });
    let fuel_for_mean_ceil = local.iter().fold(0, |acc, o| {
        acc + ((o - mean_ceil).abs() * ((o - mean_ceil).abs() + 1) / 2)
    });
    fuel_for_mean.min(fuel_for_mean_ceil)
}
fn main() -> anyhow::Result<()> {
    let input = read_to_string("day-07/input.txt").unwrap();
    let offsets = parse(&input)?;
    let part1 = alignment_cost_constant(&offsets);
    println!("part1 result is {}", part1);
    let part2 = alignment_cost_progressive(&offsets);
    println!("part2 result is {}", part2);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const INPUT: &str = "16,1,2,0,4,2,7,1,2,14";
    #[test]
    fn part1() -> anyhow::Result<()> {
        let offsets = parse(INPUT)?;
        assert_eq!(alignment_cost_constant(&offsets), 37);
        Ok(())
    }
    #[test]
    fn part2() -> anyhow::Result<()> {
        let offsets = parse(INPUT)?;
        assert_eq!(alignment_cost_progressive(&offsets), 168);
        Ok(())
    }
}
