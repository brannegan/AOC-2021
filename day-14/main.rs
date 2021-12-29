use itertools::{Itertools, MinMaxResult};
use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, line_ending},
    combinator::map,
    multi::separated_list1,
    sequence::separated_pair,
    Finish, Parser,
};
use std::{collections::HashMap, fs::read_to_string, usize};

#[derive(Debug, Clone)]
struct Polymerization {
    formula: String,
    rules: HashMap<[char; 2], char>,
    pair_count: HashMap<[char; 2], usize>,
}

impl Polymerization {
    fn new(formula: &str, rules: Vec<(&str, &str)>) -> Self {
        let rules = rules
            .iter()
            .map(|(k, v)| {
                let mut key_it = k.chars();
                (
                    [key_it.next().unwrap(), key_it.next().unwrap()],
                    v.chars().next().unwrap(),
                )
            })
            .collect::<HashMap<[char; 2], char>>();

        let mut pair_count = HashMap::new();
        for (c1, c2) in formula.chars().tuple_windows() {
            pair_count
                .entry([c1, c2])
                .and_modify(|e| *e += 1)
                .or_insert(1);
        }

        Polymerization {
            formula: formula.to_owned(),
            rules,
            pair_count,
        }
    }
    fn step_naive(&mut self) {
        let mut new_formula = String::new();
        new_formula.extend(
            self.formula
                .chars()
                .tuple_windows()
                .map(|(c1, c2)| [c1, self.rules[(&[c1, c2])]])
                .flatten()
                .chain(self.formula.chars().rev().take(1)),
        );
        self.formula = new_formula;
    }
    fn step_counting_pairs(&mut self) {
        let pair_count_new = self.pair_count.clone();
        for (k @ [first, second], count) in pair_count_new.iter().filter(|(_, v)| **v != 0) {
            let mid = self.rules[k];
            self.pair_count
                .entry([*first, mid])
                .and_modify(|e| *e += *count)
                .or_insert(*count);
            self.pair_count
                .entry([mid, *second])
                .and_modify(|e| *e += *count)
                .or_insert(*count);
            self.pair_count.entry(*k).and_modify(|e| *e -= *count);
        }
    }
}

fn parse(input: &str) -> anyhow::Result<Polymerization> {
    let formula = alpha1.map(|s: &str| s.to_owned());
    let gap = line_ending.and(line_ending);
    let rule = separated_pair(alpha1, tag(" -> "), alpha1);
    let rules = separated_list1(line_ending, rule);

    let mut parser = map(formula.and(gap).and(rules), |((formula, _), rules)| {
        Polymerization::new(&formula, rules)
    });
    parser
        .parse(input)
        .finish()
        .map(|(_input, parsed)| parsed)
        .map_err(|e: nom::error::VerboseError<&str>| anyhow::anyhow!("parser error: {:?}", e))
}
fn main() -> anyhow::Result<()> {
    let input = read_to_string("day-14/input.txt")?;

    let mut polymers = parse(&input)?;
    (0..10).for_each(|_| polymers.step_naive());
    let part1 = polymers_stat_naive(&polymers);
    println!("part1 result is {}", part1);

    let mut polymers = parse(&input)?;
    (0..40).for_each(|_| polymers.step_counting_pairs());
    let part2 = polymers_stat_counting(&polymers);
    println!("part2 result is {}", part2);
    Ok(())
}

fn polymers_stat_naive(polymers: &Polymerization) -> usize {
    let counts = polymers.formula.chars().counts();
    match counts.iter().minmax_by_key(|(_, count)| *count) {
        MinMaxResult::MinMax(x, y) => y.1 - x.1,
        _ => 0,
    }
}
fn polymers_stat_counting(polymers: &Polymerization) -> usize {
    let pair_count = &polymers.pair_count;
    let mut char_count = HashMap::<char, usize>::new();
    for (p, c) in pair_count {
        char_count.entry(p[0]).and_modify(|e| *e += c).or_insert(*c);
        char_count.entry(p[1]).and_modify(|e| *e += c).or_insert(*c);
    }
    char_count.values_mut().for_each(|v| *v /= 2);
    let fst = polymers.formula.chars().next().expect("formula not empty");
    let lst = polymers.formula.chars().last().expect("formula not empty");
    char_count.entry(fst).and_modify(|e| *e += 1);
    char_count.entry(lst).and_modify(|e| *e += 1);

    match char_count.iter().minmax_by_key(|(_, count)| *count) {
        MinMaxResult::MinMax(x, y) => y.1 - x.1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const INPUT: &str = r#"NNCB

CH -> B
HH -> N
CB -> H
NH -> C
HB -> C
HC -> B
HN -> C
NN -> C
BH -> H
NC -> B
NB -> B
BN -> B
BB -> N
BC -> B
CC -> N
CN -> C"#;
    #[test]
    fn part1() -> anyhow::Result<()> {
        let mut polymers = parse(INPUT)?;
        (0..10).for_each(|_| polymers.step_naive());
        assert_eq!(polymers_stat_naive(&polymers), 1588);

        let mut polymers = parse(INPUT)?;
        (0..10).for_each(|_| polymers.step_counting_pairs());
        assert_eq!(polymers_stat_counting(&polymers), 1588);
        Ok(())
    }
    #[test]
    fn part2() -> anyhow::Result<()> {
        let mut polymers = parse(INPUT)?;
        (0..40).for_each(|_| polymers.step_counting_pairs());
        assert_eq!(polymers_stat_counting(&polymers), 2188189693529);
        Ok(())
    }
}
