use bitvec::{bitvec, field::BitField, order::Msb0, vec::BitVec};
use nom::character::complete::line_ending;
use nom::combinator::map;
use nom::multi::{many0, separated_list0};
use nom::sequence::tuple;
use nom::{character::complete::char, Finish, Parser};
use std::fmt::{Display, Write};
use std::fs::read_to_string;
use std::iter::repeat;

#[derive(Debug, Clone)]
struct Image {
    data: Vec<BitVec>,
}
impl Image {
    fn bordered(&self, border_size: usize) -> Image {
        let w = self.data[0].len();
        let h = self.data.len();
        let mut data = Vec::with_capacity(h + 2 * border_size);
        let zeroes = BitVec::repeat(false, w + 2 * border_size);
        data.extend(repeat(zeroes.clone()).take(border_size));
        for bv in &self.data {
            let mut bordered = bv.clone();
            bordered.resize(w + 2 * border_size, false);
            bordered.shift_right(border_size);
            data.push(bordered);
        }
        data.extend(repeat(zeroes).take(border_size));
        Image { data }
    }
}
impl Display for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for bv in self.data.iter() {
            bv.iter()
                .map(|b| match *b {
                    true => '#',
                    false => '.',
                })
                .for_each(|c| f.write_char(c).unwrap());
            writeln!(f).unwrap();
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct ParsedInput {
    ieas: BitVec,
    input: Image,
}

fn parse(input: &str) -> anyhow::Result<ParsedInput> {
    let string = |i| {
        map(many0(char('.').or(char('#'))), |string| {
            string
                .into_iter()
                .map(|c| match c {
                    '.' => false,
                    '#' => true,
                    _ => unimplemented!(),
                })
                .collect()
        })(i)
    };
    let gap = line_ending.and(line_ending);
    let image = separated_list0(line_ending, string);
    let mut parser = map(tuple((string, gap, image)), |(ieas, _, data)| ParsedInput {
        ieas,
        input: Image { data },
    });
    parser
        .parse(input)
        .finish()
        .map(|(_input, parsed)| parsed)
        .map_err(|e: nom::error::VerboseError<&str>| anyhow::anyhow!("parser error: {:?}", e))
}
fn image_enhancement_algo(image: &Image, ieas: &BitVec, kernel_size: usize) -> Image {
    let width = image.data[0].len();
    let mut data = Vec::with_capacity(image.data.len());
    for rows in image.data.windows(kernel_size) {
        let mut row = BitVec::new();
        for i in 0..=width - kernel_size {
            let idx: usize = rows
                .iter()
                .fold(
                    bitvec![u16, Msb0; 0; 16 - kernel_size * kernel_size],
                    |mut acc, bv| {
                        acc.extend_from_bitslice(&bv[i..i + kernel_size]);
                        acc
                    },
                )
                .load_be();
            row.push(ieas[idx]);
        }
        data.push(row);
    }
    Image { data }
}
fn lit_pixels(image: &Image) -> usize {
    image.data.iter().fold(0, |acc, bv| acc + bv.count_ones())
}

fn main() -> anyhow::Result<()> {
    let input = read_to_string("day-20/input.txt")?;
    let parsed = parse(input.trim())?;
    let runs = 2;
    let kernel_size = 3;
    let mut image = parsed.input.bordered(runs * (kernel_size - 1));
    for _ in 0..runs {
        image = image_enhancement_algo(&image, &parsed.ieas, kernel_size);
    }
    let part1 = lit_pixels(&image);
    println!("part1 result is {part1}");

    let runs = 50;
    let kernel_size = 3;
    let mut image = parsed.input.bordered(runs * (kernel_size - 1));
    for _ in 0..runs {
        image = image_enhancement_algo(&image, &parsed.ieas, kernel_size);
    }
    let part2 = lit_pixels(&image);
    println!("part2 result is {part2}");
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    const INPUT: &str = r#"..#.#..#####.#.#.#.###.##.....###.##.#..###.####..#####..#....#..#..##..###..######.###...####..#..#####..##..#.#####...##.#.#..#.##..#.#......#.###.######.###.####...#.##.##..#..#..#####.....#.#....###..#.##......#.....#..#..#..##..#...##.######.####.####.#.#...#.......#..#.#.#...####.##.#......#..#...##.#.##..#...##.#.##..###.#......#.#.......#.#.#.####.###.##...#.....####.#..#..#.##.#....##..#.####....##...##..#...#......#.#.......#.......##..####..#...#.#.#...##..#.#..###..#####........#..####......#..#

#..#.
#....
##..#
..#..
..###"#;

    #[test]
    fn part1() -> anyhow::Result<()> {
        let parsed = parse(INPUT)?;
        let runs = 2;
        let kernel_size = 3;
        let mut image = parsed.input.bordered(runs * (kernel_size - 1));
        for _ in 0..runs {
            image = image_enhancement_algo(&image, &parsed.ieas, kernel_size);
        }
        eprintln!("{}", image.bordered(2));
        assert_eq!(lit_pixels(&image), 35);
        Ok(())
    }
    #[test]
    fn part2() -> anyhow::Result<()> {
        let parsed = parse(INPUT)?;
        let runs = 50;
        let kernel_size = 3;
        let mut image = parsed.input.bordered(runs * (kernel_size - 1));
        for _ in 0..runs {
            image = image_enhancement_algo(&image, &parsed.ieas, kernel_size);
        }
        eprintln!("{}", image);
        assert_eq!(lit_pixels(&image), 3351);
        Ok(())
    }
}

