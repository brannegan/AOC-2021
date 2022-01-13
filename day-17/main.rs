use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::i32,
    sequence::{preceded, separated_pair},
    Finish, Parser,
};
use std::{collections::HashSet, fs::read_to_string, ops::RangeInclusive};

#[derive(Debug, Clone)]
struct TargetArea {
    x_range: RangeInclusive<i32>,
    y_range: RangeInclusive<i32>,
}

fn parse(input: &str) -> anyhow::Result<TargetArea> {
    let x_range = preceded(tag("x="), separated_pair(i32, tag(".."), i32));
    let y_range = preceded(tag("y="), separated_pair(i32, tag(".."), i32));
    let ranges = separated_pair(x_range, tag(", "), y_range);
    let mut parser = preceded(tag("target area: "), ranges).map(|(x, y)| TargetArea {
        x_range: x.0..=x.1,
        y_range: y.0..=y.1,
    });
    parser
        .parse(input)
        .finish()
        .map(|(_input, parsed)| parsed)
        .map_err(|e: nom::error::VerboseError<&str>| anyhow::anyhow!("parser error: {:?}", e))
}
fn max_height(ta: TargetArea) -> i32 {
    let vels = possible_velocities(ta);
    let max_y = vels
        .iter()
        .max_by(|a, b| a.1.cmp(&b.1))
        .map(|(_, y)| y)
        .expect("max found");
    max_y * (max_y + 1) / 2
}
fn possible_velocities_count(ta: TargetArea) -> usize {
    possible_velocities(ta).len()
}
fn possible_velocities(ta: TargetArea) -> HashSet<(i32, i32)> {
    let mut vel_xs: Vec<RangeInclusive<i32>> = (1..)
        .map(|n| {
            let left = ta.x_range.start() + n * (n - 1) / 2;
            let right = ta.x_range.end() + n * (n - 1) / 2;
            let ceil = (left.rem_euclid(n) != 0) as i32;
            let start_vel_x_range = (left.div_euclid(n) + ceil)..=right.div_euclid(n);
            (n, start_vel_x_range)
        })
        .take_while(|(n, _)| *ta.x_range.start() > n * (n - 1) / 2)
        .map(|(_, start_vel_x_range)| start_vel_x_range)
        .collect();
    let vel_ys: Vec<RangeInclusive<i32>> = (1i32..)
        .map(|n| {
            let bottom = ta.y_range.start() + n * (n - 1) / 2;
            let top = ta.y_range.end() + n * (n - 1) / 2;
            let ceil = (bottom.rem_euclid(n) != 0) as i32;
            let start_vel_y_range = (bottom.div_euclid(n) + ceil)..=top.div_euclid(n);
            (n, start_vel_y_range)
        })
        .take_while(|(n, start_vel_y_range)| {
            let end_vel = start_vel_y_range.end() + (n - 1) * (-1);
            end_vel >= *ta.y_range.start()
        })
        .map(|(_, start_vel_y_range)| start_vel_y_range)
        .collect();
    let last_x = vel_xs.last().unwrap().clone();
    vel_xs.resize(vel_ys.len(), last_x);
    vel_xs
        .into_iter()
        .zip(vel_ys.into_iter())
        .flat_map(move |(xr, yr)| xr.cartesian_product(yr.into_iter()))
        .collect()
}
fn main() -> anyhow::Result<()> {
    let input = read_to_string("day-17/input.txt")?;
    let target_area = parse(&input)?;
    let part1 = max_height(target_area.clone());
    assert_eq!(part1, 4095);
    println!("part1 result is {}", part1);
    let part2 = possible_velocities_count(target_area);
    assert_eq!(part2, 3773);
    println!("part2 result is {}", part2);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const INPUT: &str = "target area: x=20..30, y=-10..-5";

    #[test]
    fn part1() -> anyhow::Result<()> {
        let target_area = parse(INPUT)?;
        assert_eq!(max_height(target_area), 45);
        Ok(())
    }
    #[test]
    fn part2() -> anyhow::Result<()> {
        let target_area = parse(INPUT)?;
        assert_eq!(possible_velocities_count(target_area), 112);
        Ok(())
    }
}
