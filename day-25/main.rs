use anyhow::Ok;

use nom::{
    character::complete::{line_ending, one_of},
    multi::{many0, separated_list1},
    Finish, Parser,
};
use std::fs::read_to_string;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cucumber {
    EastFacing,
    SouthFacing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Occupied(Cucumber),
    Empty,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct Map {
    tiles: Vec<Vec<Tile>>,
}
impl Map {
    fn new(tiles: Vec<Vec<Tile>>) -> Self {
        Self { tiles }
    }
    fn step(self) -> Self {
        self.step_hor().step_vert()
    }

    fn step_hor(self) -> Self {
        let w = self.tiles[0].len();
        let h = self.tiles.len();
        let mut tiles = vec![vec![Tile::Empty; w]; h];
        for (r, row) in self.tiles.iter().enumerate() {
            for (c, &tile) in row.iter().enumerate() {
                tiles[r][c] = if tile == Tile::Occupied(Cucumber::EastFacing)
                    && row[(c + 1) % w] == Tile::Empty
                {
                    Tile::Empty
                } else if row[if c == 0 { w - 1 } else { c - 1 }]
                    == Tile::Occupied(Cucumber::EastFacing)
                    && tile == Tile::Empty
                {
                    Tile::Occupied(Cucumber::EastFacing)
                } else {
                    tile
                }
            }
        }
        Self { tiles }
    }
    fn step_vert(self) -> Self {
        let w = self.tiles[0].len();
        let h = self.tiles.len();
        let mut tiles = vec![vec![Tile::Empty; w]; h];
        for (r, row) in self.tiles.iter().enumerate() {
            for (c, &tile) in row.iter().enumerate() {
                tiles[r][c] = if tile == Tile::Occupied(Cucumber::SouthFacing)
                    && self.tiles[(r + 1) % h][c] == Tile::Empty
                {
                    Tile::Empty
                } else if self.tiles[if r == 0 { h - 1 } else { r - 1 }][c]
                    == Tile::Occupied(Cucumber::SouthFacing)
                    && tile == Tile::Empty
                {
                    Tile::Occupied(Cucumber::SouthFacing)
                } else {
                    tile
                }
            }
        }
        Self { tiles }
    }
    fn iter(self) -> MapIter {
        MapIter { map: Some(self) }
    }
}
struct MapIter {
    map: Option<Map>,
}
impl Iterator for MapIter {
    type Item = Map;

    fn next(&mut self) -> Option<Self::Item> {
        self.map.as_ref()?;
        let cur = self.map.take();
        let new = cur.clone().map(Map::step);
        (new != cur).then(|| self.map = new);
        cur
    }
}

impl std::fmt::Display for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.tiles {
            for tile in row {
                write!(
                    f,
                    "{}",
                    match tile {
                        Tile::Occupied(Cucumber::EastFacing) => '>',
                        Tile::Occupied(Cucumber::SouthFacing) => 'v',
                        Tile::Empty => '.',
                    },
                )?;
            }
            writeln!(f)?;
        }
        std::fmt::Result::Ok(())
    }
}

fn parse(input: &str) -> anyhow::Result<Map> {
    let tile = one_of("v>.").map(|c| match c {
        '>' => Tile::Occupied(Cucumber::EastFacing),
        'v' => Tile::Occupied(Cucumber::SouthFacing),
        '.' => Tile::Empty,
        _ => unimplemented!(),
    });
    let mut parser = separated_list1(line_ending, many0(tile)).map(Map::new);
    parser
        .parse(input)
        .finish()
        .map(|(_input, parsed)| parsed)
        .map_err(|e: nom::error::VerboseError<&str>| anyhow::anyhow!("parser error: {:?}", e))
}
fn main() -> anyhow::Result<()> {
    let map = parse(read_to_string("day-25/input.txt")?.trim())?;
    let (i, map) = map.iter().enumerate().last().unwrap();
    println!("turns {}\n{}", i + 1, map);
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    const INPUT: &str = r#"
v...>>.vv>
.vv>>.vv..
>>.>v>...v
>>v>>.>.v.
v>v.vv.v..
>.>>..v...
.vv..>.>v.
v.v..>>v.v
....v..v.>
    "#;
    const INPUT1: &str = r#"
....>.>v.>
v.v>.>v.v.
>v>>..>v..
>>v>v>.>.v
.>v.v...v.
v>>.>vvv..
..v...>>..
vv...>>vv.
>.v.v..v.v
    "#;
    const INPUT2: &str = r#"
>.v.v>>..v
v.v.>>vv..
>v>.>.>.v.
>>v>v.>v>.
.>..v....v
.>v>>.v.v.
v....v>v>.
.vv..>>v..
v>.....vv.
    "#;

    #[test]
    fn step() -> anyhow::Result<()> {
        assert_eq!(parse(">...>>...>")?.step_hor(), parse(".>..>.>..>")?);
        let map = parse(INPUT.trim())?;
        let map = map.step();
        assert_eq!(parse(INPUT1.trim())?, map);

        let map = map.step();
        assert_eq!(parse(INPUT2.trim())?, map);

        Ok(())
    }
    #[test]
    fn part1() -> anyhow::Result<()> {
        let map = parse(INPUT.trim())?;
        assert_eq!(map.iter().count(), 58);
        Ok(())
    }
}
