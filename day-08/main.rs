use nom::{
    character::complete::{alpha1, char, line_ending, multispace1, space1},
    combinator::map,
    multi::separated_list1,
    sequence::separated_pair,
    Finish, Parser,
};
use std::{
    collections::{BTreeSet, HashMap},
    fs::read_to_string,
};

#[derive(Debug, Clone)]
struct ParsedInput {
    mapping: HashMap<SegmentPatterns, DigitalOutput>,
}
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct SegmentPatterns {
    patterns: Vec<BTreeSet<char>>,
}
#[derive(Debug, Clone)]
struct DigitalOutput {
    digits: Vec<BTreeSet<char>>,
}
fn parse(input: &str) -> anyhow::Result<ParsedInput> {
    let words = |i| {
        separated_list1(
            space1,
            alpha1.map(|w: &str| w.chars().collect::<BTreeSet<_>>()),
        )(i)
    };
    let entry = |i| {
        separated_pair(
            words.map(|patterns| SegmentPatterns { patterns }),
            space1.and(char('|')).and(multispace1),
            words.map(|output| DigitalOutput { digits: output }),
        )(i)
    };
    let entries = separated_list1(line_ending, entry);
    let mut parser = map(entries, |entries| ParsedInput {
        mapping: entries.into_iter().collect(),
    });
    parser
        .parse(input)
        .finish()
        .map(|(_input, parsed)| parsed)
        .map_err(|_: nom::error::Error<&str>| anyhow::anyhow!("parser error"))
}
fn main() -> anyhow::Result<()> {
    let input = read_to_string("day-08/input.txt").unwrap();
    let parsed = parse(&input)?;
    let part1 = count_digits(&parsed);
    println!("part1 result is {}", part1);
    let part2 = decode_digits(&parsed);
    println!("part2 result is {}", part2);
    Ok(())
}

fn count_digits(input: &ParsedInput) -> i32 {
    input
        .mapping
        .values()
        .map(|outputs| &outputs.digits)
        .flatten()
        .filter(|o| [2, 4, 3, 7].contains(&o.len()))
        .count() as i32
}
fn decode_digits(input: &ParsedInput) -> u32 {
    input
        .mapping
        .iter()
        .map(|(segment_pattern, digit_output)| {
            let four = segment_pattern
                .patterns
                .iter()
                .find(|pat| pat.len() == 4)
                .unwrap();
            let seven = segment_pattern
                .patterns
                .iter()
                .find(|pat| pat.len() == 3)
                .unwrap();
            let number_len = digit_output.digits.len() as u32 - 1;
            digit_output
                .digits
                .iter()
                .map(|o| {
                    (
                        o.len(),
                        o.intersection(four).count(),
                        o.intersection(seven).count(),
                    )
                })
                .enumerate()
                .fold(0u32, |acc, (i, deduct_rules)| {
                    let digit = match deduct_rules {
                        (2, _, _) => 1,
                        (3, _, _) => 7,
                        (4, _, _) => 4,
                        (5, 2, 2) => 2,
                        (5, 3, 3) => 3,
                        (5, 3, 2) => 5,
                        (6, 3, 3) => 0,
                        (6, 3, 2) => 6,
                        (6, 4, 3) => 9,
                        (7, _, _) => 8,
                        _ => unreachable!(),
                    };
                    acc + digit * 10_u32.pow(number_len - i as u32)
                })
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"be cfbegad cbdgef fgaecd cgeb fdcge agebfd fecdb fabcd edb |
fdgacbe cefdb cefbgd gcbe
edbfga begcd cbg gc gcadebf fbgde acbgfd abcde gfcbed gfec |
fcgedb cgb dgebacf gc
fgaebd cg bdaec gdafb agbcfd gdcbef bgcad gfac gcb cdgabef |
cg cg fdcagb cbg
fbegcd cbd adcefb dageb afcb bc aefdc ecdab fgdeca fcdbega |
efabcd cedba gadfec cb
aecbfdg fbg gf bafeg dbefa fcge gcbea fcaegb dgceab fcbdga |
gecf egdcabf bgf bfgea
fgeab ca afcebg bdacfeg cfaedg gcfdb baec bfadeg bafgc acf |
gebdcfa ecba ca fadegcb
dbcfg fgd bdegcaf fgec aegbdf ecdfab fbedc dacgb gdcebf gf |
cefg dcbef fcge gbcadfe
bdfegc cbegaf gecbf dfcage bdacg ed bedf ced adcbefg gebcd |
ed bcgafe cdgba cbgef
egadfb cdbfeg cegd fecab cgb gbdefca cg fgcdab egfdb bfceg |
gbdfcae bgc cg cgb
gcafb gcf dcaebfg ecagb gf abcdeg gaef cafbge fdbac fegbdc |
fgae cfgab fg bagce
"#;
    #[test]
    fn part1() -> anyhow::Result<()> {
        let parsed = parse(INPUT)?;
        assert_eq!(count_digits(&parsed), 26);
        Ok(())
    }
    #[test]
    fn part2() -> anyhow::Result<()> {
        let parsed = parse(INPUT)?;
        assert_eq!(decode_digits(&parsed), 61229);
        Ok(())
    }
}
