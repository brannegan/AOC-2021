use itertools::Itertools;
use nom::{
    bytes::complete::take,
    character::complete::{line_ending, u32},
    multi::{many1, separated_list1},
    Finish, Parser,
};
use petgraph::{algo::astar, graph::NodeIndex, Graph};
use std::fs::read_to_string;

#[derive(Debug, Clone)]
struct RiskMap {
    graph: Graph<(), u32>,
}

impl RiskMap {
    fn new(arr: Vec<Vec<u32>>) -> Self {
        let mut graph = Graph::new();
        let w = arr[0].len();
        let h = arr.len();

        (0..h).cartesian_product(0..w).for_each(|_| {
            graph.add_node(());
        });
        let adj = |i: usize, j: usize| {
            let left = i.checked_sub(1).map(|i| (i, j));
            let up = j.checked_sub(1).map(|j| (i, j));
            let right = (i + 1 < w).then(|| (i + 1, j));
            let down = (j + 1 < h).then(|| (i, j + 1));
            [left, right, up, down]
        };
        (0..h).cartesian_product(0..w).for_each(|(i, j)| {
            adj(i, j).iter().filter_map(|e| *e).for_each(|(x, y)| {
                graph.add_edge(
                    NodeIndex::new(i + h * j),
                    NodeIndex::new(x + h * y),
                    arr[x][y],
                );
            });
        });
        Self { graph }
    }
    fn a_star_path(&self) -> Option<u32> {
        let (cost, _path) = astar(
            &self.graph,
            NodeIndex::new(0),
            |finish| finish == NodeIndex::new(self.graph.node_count() - 1),
            |e| *e.weight(),
            |_| 0,
        )?;
        Some(cost)
    }
}

fn parse(input: &str) -> anyhow::Result<Vec<Vec<u32>>> {
    let risk = take(1usize).and_then(u32);
    let line = many1(risk);
    let mut parser = separated_list1(line_ending, line);
    parser
        .parse(input)
        .finish()
        .map(|(_input, parsed)| parsed)
        .map_err(|e: nom::error::VerboseError<&str>| anyhow::anyhow!("parser error: {:?}", e))
}
fn tiled(tile: Vec<Vec<u32>>, num: usize) -> Vec<Vec<u32>> {
    let w = tile[0].len();
    let h = tile.len();
    let mut res = vec![vec![0; w * num]; h * num];
    for i in 0..h * num {
        for j in 0..w * num {
            let shift_and_inc = (i / h) as isize + (j / w) as isize - 1;
            res[i][j] = ((tile[i % h][j % w] as isize + shift_and_inc) % 9 + 1) as u32;
        }
    }
    res
}
fn main() -> anyhow::Result<()> {
    let input = read_to_string("day-15/input.txt")?;
    let parsed = parse(&input)?;
    let risk_map = RiskMap::new(parsed.clone());
    let tiled_map = tiled(parsed, 5);
    let tiled_map = RiskMap::new(tiled_map);
    let part1 = path_risk_level(&risk_map);
    println!("part1 result is {}", part1);
    let part2 = path_risk_level(&tiled_map);
    println!("part2 result is {}", part2);
    Ok(())
}

fn path_risk_level(input: &RiskMap) -> u32 {
    input.a_star_path().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    const INPUT: &str = r#"1163751742
1381373672
2136511328
3694931569
7463417111
1319128137
1359912421
3125421639
1293138521
2311944581"#;
    #[test]
    fn part1() -> anyhow::Result<()> {
        let parsed = parse(INPUT)?;
        let risk_map = RiskMap::new(parsed);
        assert_eq!(path_risk_level(&risk_map), 40);

        Ok(())
    }
    #[test]
    fn part2() -> anyhow::Result<()> {
        let parsed = parse(INPUT)?;
        let tiled_map = tiled(parsed, 5);
        let tiled_map = RiskMap::new(tiled_map);
        assert_eq!(path_risk_level(&tiled_map), 315);
        Ok(())
    }
}
