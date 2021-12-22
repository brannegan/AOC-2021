use nom::{
    character::complete::{line_ending, one_of},
    combinator::map,
    multi::{many1, separated_list1},
    Finish, Parser,
};
use std::{fs::read_to_string, ops::Not};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Bracket {
    kind: BracketKind,
    side: BracketSide,
}
impl Bracket {
    fn score_corrupted(&self) -> u64 {
        use crate::{BracketKind::*, BracketSide::*};
        match self {
            Bracket {
                kind: Paren,
                side: Close,
            } => 3,
            Bracket {
                kind: Square,
                side: Close,
            } => 57,
            Bracket {
                kind: Curly,
                side: Close,
            } => 1197,
            Bracket {
                kind: Angle,
                side: Close,
            } => 25137,
            _ => 0,
        }
    }
    fn score_incomplete(&self) -> u64 {
        use crate::{BracketKind::*, BracketSide::*};
        match self {
            Bracket {
                kind: Paren,
                side: Close,
            } => 1,
            Bracket {
                kind: Square,
                side: Close,
            } => 2,
            Bracket {
                kind: Curly,
                side: Close,
            } => 3,
            Bracket {
                kind: Angle,
                side: Close,
            } => 4,
            _ => 0,
        }
    }
    fn is_balanced(&self, other: &Bracket) -> bool {
        self.kind == other.kind
            && self.side == BracketSide::Open
            && other.side == BracketSide::Close
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BracketKind {
    Paren,
    Square,
    Curly,
    Angle,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BracketSide {
    Open,
    Close,
}

#[derive(Debug, Clone)]
struct ParsedInput {
    chunks: Vec<Vec<Bracket>>,
}
fn parse(input: &str) -> anyhow::Result<ParsedInput> {
    use crate::{BracketKind::*, BracketSide::*};
    let bracket = one_of("([{<>}])").map(|c| match c {
        '(' => Bracket {
            kind: Paren,
            side: Open,
        },
        '[' => Bracket {
            kind: Square,
            side: Open,
        },
        '{' => Bracket {
            kind: Curly,
            side: Open,
        },
        '<' => Bracket {
            kind: Angle,
            side: Open,
        },
        '>' => Bracket {
            kind: Angle,
            side: Close,
        },
        '}' => Bracket {
            kind: Curly,
            side: Close,
        },
        ']' => Bracket {
            kind: Square,
            side: Close,
        },
        ')' => Bracket {
            kind: Paren,
            side: Close,
        },
        _ => unreachable!(),
    });
    let chunk = many1(bracket);
    let chunks = separated_list1(line_ending, chunk);
    let mut parser = map(chunks, |chunks| ParsedInput { chunks });
    parser
        .parse(input)
        .finish()
        .map(|(_input, parsed)| parsed)
        .map_err(|_: nom::error::Error<&str>| anyhow::anyhow!("parser error"))
}
fn main() -> anyhow::Result<()> {
    let input = read_to_string("day-10/input.txt")?;
    let parsed = parse(&input)?;
    let part1 = corrupted_chunks_score(&parsed);
    println!("part1 result is {}", part1);
    let part2 = incomplete_chunks_middle_score(&parsed);
    println!("part2 result is {}", part2);
    Ok(())
}

fn corrupted_chunks_score(input: &ParsedInput) -> u64 {
    let mut queue = Vec::new();
    input
        .chunks
        .iter()
        .filter_map(|chunk| {
            queue.clear();
            for bracket in chunk {
                match bracket {
                    Bracket {
                        side: BracketSide::Open,
                        ..
                    } => queue.push(bracket),
                    Bracket {
                        side: BracketSide::Close,
                        ..
                    } => {
                        if !queue.pop().unwrap().is_balanced(bracket) {
                            return Some(bracket);
                        }
                    }
                }
            }
            None
        })
        .fold(0, |acc, bracket| acc + bracket.score_corrupted())
}

fn incomplete_chunks_middle_score(input: &ParsedInput) -> u64 {
    let mut incomplete_scores: Vec<u64> = input
        .chunks
        .iter()
        .filter_map(|chunk| {
            let mut queue = Vec::new();
            for bracket in chunk {
                match bracket {
                    Bracket {
                        side: BracketSide::Open,
                        ..
                    } => queue.push(bracket),
                    Bracket {
                        side: BracketSide::Close,
                        ..
                    } => {
                        if !queue.pop().unwrap().is_balanced(bracket) {
                            return None;
                        }
                    }
                }
            }
            queue.is_empty().not().then(|| queue)
        })
        .map(|incomplete| {
            incomplete
                .into_iter()
                .rev()
                .map(|bracket| Bracket {
                    kind: bracket.kind,
                    side: BracketSide::Close,
                })
                .fold(0, |acc, bracket| acc * 5 + bracket.score_incomplete())
        })
        .collect();
    incomplete_scores.sort_unstable();
    *incomplete_scores.get(incomplete_scores.len() / 2).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"[({(<(())[]>[[{[]{<()<>>
[(()[<>])]({[<{<<[]>>(
{([(<{}[<>[]}>{[]{[(<()>
(((({<>}<{<{<>}{[]{[]{}
[[<[([]))<([[{}[[()]]]
[{[{({}]{}}([{[{{{}}([]
{<[[]]>}<{[{[{[]{()[[[]
[<(<(<(<{}))><([]([]()
<{([([[(<>()){}]>(<<{{
<{([{{}}[<[[[<>{}]]]>[]]
"#;
    #[test]
    fn part1() -> anyhow::Result<()> {
        let parsed = parse(INPUT)?;
        assert_eq!(corrupted_chunks_score(&parsed), 26397);
        Ok(())
    }
    #[test]
    fn part2() -> anyhow::Result<()> {
        let parsed = parse(INPUT)?;
        assert_eq!(incomplete_chunks_middle_score(&parsed), 288957);
        Ok(())
    }
}
