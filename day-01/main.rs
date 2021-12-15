use std::fs::read_to_string;

use itertools::Itertools;

fn parse(input: &str) -> Vec<u32> {
    input
        .trim()
        .split('\n')
        .map(|n| n.parse::<u32>())
        .map(Result::unwrap)
        .collect()
}
fn count_increased(depths: &[u32]) -> usize {
    depths
        .iter()
        .tuple_windows()
        .filter(|(n1, n2)| n2 > n1)
        .count()
}
fn sliding_windows(depths: &[u32]) -> usize {
    depths
        .iter()
        .tuple_windows()
        .map(|(n1, n2, n3)| n3 + n2 + n1)
        .tuple_windows()
        .filter(|(n1, n2)| n2 > n1)
        .count()
}
fn main() {
    let input = read_to_string("day-01/input.txt").unwrap();
    let depths = parse(&input);
    let part1 = count_increased(&depths);
    let part2 = sliding_windows(&depths);
    println!("part1 result is {}", part1);
    println!("part2 result is {}", part2);
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"199
200
208
210
200
207
240
269
260
263"#;
    #[test]
    fn part1() {
        assert_eq!(count_increased(&parse(INPUT)), 7);
    }
    #[test]
    fn part2() {
        assert_eq!(sliding_windows(&parse(INPUT)), 5);
    }
}
