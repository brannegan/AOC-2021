use ndarray::{s, Array2};
use nom::{
    bytes::complete::tag,
    character::complete::{char, line_ending, one_of, u32},
    combinator::map,
    multi::separated_list1,
    sequence::separated_pair,
    Finish, Parser,
};
use std::fs::read_to_string;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Fold {
    X(u32),
    Y(u32),
}
#[derive(Debug, Clone)]
struct PaperFolds {
    paper: Array2<u32>,
    folds: Vec<Fold>,
}

impl PaperFolds {
    fn new(dots: Vec<(u32, u32)>, folds: Vec<Fold>) -> Self {
        let (bound_x, bound_y) = dots.iter().fold((0, 0), |(max_x, max_y), (x, y)| {
            (max_x.max(*x), max_y.max(*y))
        });
        let mut paper = Array2::zeros((bound_y as usize + 1, bound_x as usize + 1));
        dots.iter()
            .copied()
            .for_each(|(x, y)| paper[(y as usize, x as usize)] = 1);
        Self { paper, folds }
    }
    fn apply(&mut self, fold: Fold) {
        match fold {
            Fold::X(x_pos) => {
                let x_pos = x_pos as usize;
                let (mut v_left, v_right) = self.paper.view_mut().split_at(ndarray::Axis(1), x_pos);
                v_left.zip_mut_with(&v_right.slice(s![.., 1..;-1]), |u, d| *u |= *d);
                self.paper = v_left.into_owned();
            }
            Fold::Y(y_pos) => {
                let y_pos = y_pos as usize;
                let (mut v_up, v_down) = self.paper.view_mut().split_at(ndarray::Axis(0), y_pos);
                v_up.zip_mut_with(&v_down.slice(s![1..;-1, ..]), |u, d| *u |= *d);
                self.paper = v_up.into_owned();
            }
        }
    }
}

fn parse(input: &str) -> anyhow::Result<PaperFolds> {
    let dot = separated_pair(u32, char(','), u32);
    let dots = separated_list1(line_ending, dot);
    let gap = line_ending.and(line_ending);
    let fold = tag("fold along ")
        .and(separated_pair(one_of("xy"), char('='), u32))
        .map(|(_, (axis, pos))| match axis {
            'x' => Fold::X(pos),
            'y' => Fold::Y(pos),
            _ => unreachable!(),
        });
    let folds = separated_list1(line_ending, fold);

    let mut parser = map(dots.and(gap).and(folds), |((dots, _), fold)| {
        PaperFolds::new(dots, fold)
    });
    parser
        .parse(input)
        .finish()
        .map(|(_input, parsed)| parsed)
        .map_err(|e: nom::error::VerboseError<&str>| anyhow::anyhow!("parser error: {:?}", e))
}
fn main() -> anyhow::Result<()> {
    let input = read_to_string("day-13/input.txt")?;
    let mut paper_folds = parse(&input)?;
    let part1 = dots_after_one_fold(&mut paper_folds);
    println!("part1 result is {}", part1);
    letters(&mut paper_folds);
    Ok(())
}

fn dots_after_one_fold(paper_folds: &mut PaperFolds) -> u32 {
    let fold = *paper_folds.folds.first().expect("some folds");
    paper_folds.apply(fold);
    paper_folds.paper.sum()
}
fn letters(paper_folds: &mut PaperFolds) {
    let folds = paper_folds.folds.clone();
    for fold in folds.iter().skip(1) {
        paper_folds.apply(*fold);
    }
    for row in paper_folds.paper.rows() {
        let letter: String = row
            .iter()
            .map(|c| match c {
                1 => '#',
                0 => '.',
                _ => '!',
            })
            .collect();
        println!("{}", letter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"6,10
0,14
9,10
0,3
10,4
4,11
6,0
6,12
4,1
0,13
10,12
3,4
3,0
8,4
1,10
2,14
8,10
9,0

fold along y=7
fold along x=5"#;
    #[test]
    fn part1() -> anyhow::Result<()> {
        let mut paper_folds = parse(INPUT)?;
        assert_eq!(dots_after_one_fold(&mut paper_folds), 17);
        Ok(())
    }
}
