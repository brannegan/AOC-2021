use nom::{
    bytes::complete::tag,
    character::complete::{char, i32, line_ending},
    combinator::map,
    multi::separated_list1,
    sequence::separated_pair,
    Finish, Parser,
};
use std::fs::read_to_string;

#[derive(Debug, Clone, Copy)]
struct Line {
    a: (i32, i32),
    b: (i32, i32),
}
impl Line {
    fn draw(&self, diagram: &mut Vec<Vec<i32>>) {
        for point in self.iter() {
            diagram[point.1 as usize][point.0 as usize] += 1;
        }
    }
    fn hor_vert(&self) -> bool {
        let Line { a, b } = self;
        a.0 == b.0 || a.1 == b.1
    }
    fn diagonal(&self) -> bool {
        let Line { a, b } = self;
        a.0 == b.0 || a.1 == b.1 || (a.0 - b.0).abs() == (a.1 - b.1).abs()
    }
    fn iter(&self) -> LineIter {
        LineIter {
            line: self,
            cur: self.a,
            stop: false,
        }
    }
}
struct LineIter<'a> {
    line: &'a Line,
    cur: (i32, i32),
    stop: bool,
}
impl Iterator for LineIter<'_> {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.stop {
            return None;
        }
        let Line { a, b } = self.line;
        let cur = self.cur;

        if cur == *b {
            self.stop = true;
            return Some(cur);
        }
        self.cur.0 += if b.0 == a.0 {
            0
        } else {
            (b.0 - a.0).abs() / (b.0 - a.0)
        };
        self.cur.1 += if b.1 == a.1 {
            0
        } else {
            (b.1 - a.1).abs() / (b.1 - a.1)
        };
        Some(cur)
    }
}

#[derive(Debug, Clone)]
struct ParsedInput {
    lines: Vec<Line>,
}

fn parse(input: &str) -> anyhow::Result<ParsedInput> {
    let coord = |i| separated_pair(i32, char(','), i32)(i);
    let line = separated_pair(coord, tag(" -> "), coord).map(|(a, b)| Line { a, b });
    let lines = separated_list1(line_ending, line);
    let mut parser = map(lines, |lines| ParsedInput { lines });
    parser
        .parse(input)
        .finish()
        .map(|(_input, parsed)| parsed)
        .map_err(|_: nom::error::Error<&str>| anyhow::anyhow!("parser error"))
}
fn main() -> anyhow::Result<()> {
    let input = read_to_string("day-05/input.txt")?;
    let parsed = parse(&input)?;
    let part1 = lines_overlaped(&parsed, Line::hor_vert);
    println!("part1 result is {}", part1);
    let part2 = lines_overlaped(&parsed, Line::diagonal);
    println!("part2 result is {}", part2);
    Ok(())
}

fn lines_overlaped<P>(input: &ParsedInput, p: P) -> i32
where
    P: FnMut(&Line) -> bool,
{
    let lines: Vec<Line> = input.lines.iter().copied().filter(p).collect();
    let diagram_size = lines
        .iter()
        .map(|Line { a, b }| *[a.0, b.0, a.1, b.1].iter().max().unwrap())
        .max()
        .expect("input not empty") as usize
        + 1;
    let mut diagram = vec![vec![0i32; diagram_size]; diagram_size];
    for line in &lines {
        line.draw(&mut diagram);
    }
    diagram
        .iter()
        .flatten()
        .copied()
        .filter(|&e| e >= 2)
        .count() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"0,9 -> 5,9
8,0 -> 0,8
9,4 -> 3,4
2,2 -> 2,1
7,0 -> 7,4
6,4 -> 2,0
0,9 -> 2,9
3,4 -> 1,4
0,0 -> 8,8
5,5 -> 8,2
"#;
    #[test]
    fn part1() -> anyhow::Result<()> {
        let parsed = parse(INPUT)?;
        assert_eq!(lines_overlaped(&parsed, Line::hor_vert), 5);
        Ok(())
    }
    #[test]
    fn part2() -> anyhow::Result<()> {
        let parsed = parse(INPUT)?;
        assert_eq!(lines_overlaped(&parsed, Line::diagonal), 12);
        Ok(())
    }
}
