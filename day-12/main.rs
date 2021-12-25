use itertools::Itertools;
use nom::{
    character::complete::{alpha1, char, line_ending},
    combinator::map,
    multi::separated_list1,
    sequence::separated_pair,
    Finish, Parser,
};
use petgraph::{
    graph::{NodeIndex, UnGraph},
    Graph,
};
use std::{collections::HashMap, fs::read_to_string};

#[derive(Debug, Clone)]
struct Caves<'i> {
    graph: UnGraph<&'i str, ()>,
    nodes_map: HashMap<&'i str, NodeIndex>,
}

impl<'i> Caves<'i> {
    fn new(edges: Vec<(&'i str, &'i str)>) -> Self {
        let mut graph = Graph::new_undirected();
        let mut nodes_map = HashMap::new();
        for edge in edges {
            if !nodes_map.contains_key(edge.0) {
                let node_id = graph.add_node(edge.0);
                nodes_map.insert(edge.0, node_id);
            }
            if !nodes_map.contains_key(edge.1) {
                let node_id = graph.add_node(edge.1);
                nodes_map.insert(edge.1, node_id);
            }
            graph.add_edge(
                *nodes_map.get(edge.0).unwrap(),
                *nodes_map.get(edge.1).unwrap(),
                (),
            );
        }
        Self { graph, nodes_map }
    }
}
fn parse(input: &str) -> anyhow::Result<Caves> {
    let edge = separated_pair(alpha1, char('-'), alpha1);
    let edges = separated_list1(line_ending, edge);
    let mut parser = map(edges, Caves::new);
    parser
        .parse(input)
        .finish()
        .map(|(_input, parsed)| parsed)
        .map_err(|err: nom::error::VerboseError<&str>| anyhow::anyhow!("parser error: {:?}", err))
}
fn main() -> anyhow::Result<()> {
    let input = read_to_string("day-12/input.txt")?;
    let graph = parse(&input)?;
    let part1 = cave_paths(&graph, PathSelection::Once);
    println!("part1 result is {}", part1.unwrap());
    let part2 = cave_paths(&graph, PathSelection::Twice);
    println!("part2 result is {}", part2.unwrap());
    Ok(())
}

fn single_cave_once(_: &UnGraph<&str, ()>, path: &[NodeIndex], cave: NodeIndex) -> bool {
    !path.contains(&cave)
}
fn single_cave_twice(graph: &UnGraph<&str, ()>, path: &[NodeIndex], cave: NodeIndex) -> bool {
    if graph[cave] == "start" {
        return false;
    }
    if graph[cave].chars().all(|c| c.is_ascii_uppercase()) {
        return true;
    }
    !path.contains(&cave)
        || path
            .iter()
            .filter(|n| graph[**n].chars().all(|c| c.is_ascii_lowercase()))
            .all_unique()
}
fn all_paths(
    graph: &UnGraph<&str, ()>,
    from: NodeIndex,
    to: NodeIndex,
    is_path_selected: impl Fn(&UnGraph<&str, ()>, &[NodeIndex], NodeIndex) -> bool,
) -> Vec<Vec<NodeIndex>> {
    let mut paths = vec![];
    let mut stack = vec![vec![from]];
    while let Some(mut last_path) = stack.pop() {
        let last_node = last_path.last().unwrap();
        for neighbor in graph.neighbors(*last_node) {
            if neighbor == to {
                last_path.push(to);
                paths.push(last_path.clone());
                last_path.pop();
                continue;
            }
            if is_path_selected(graph, &last_path, neighbor) {
                let mut new_path = last_path.clone();
                new_path.push(neighbor);
                stack.push(new_path);
            }
        }
    }
    paths
}
enum PathSelection {
    Once,
    Twice,
}
fn cave_paths(caves: &Caves, path_selection: PathSelection) -> Option<u32> {
    let from = caves.nodes_map.get("start")?;
    let to = caves.nodes_map.get("end")?;
    let paths = match path_selection {
        PathSelection::Once => all_paths(&caves.graph, *from, *to, single_cave_once),
        PathSelection::Twice => all_paths(&caves.graph, *from, *to, single_cave_twice),
    };

    //for path in &paths {
    //    eprintln!(
    //        "path = {:?}",
    //        path.iter().map(|n| caves.graph[*n]).collect::<Vec<_>>()
    //    );
    //}
    Some(paths.len() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    const INPUT1: &str = r#"start-A
start-b
A-c
A-b
b-d
A-end
b-end"#;
    const INPUT2: &str = r#"dc-end
HN-start
start-kj
dc-start
dc-HN
LN-dc
HN-end
kj-sa
kj-HN
kj-dc"#;
    const INPUT3: &str = r#"fs-end
he-DX
fs-he
start-DX
pj-DX
end-zg
zg-sl
zg-pj
pj-he
RW-he
fs-DX
pj-RW
zg-RW
start-pj
he-WI
zg-he
pj-fs
start-RW"#;
    #[test]
    fn part1() -> anyhow::Result<()> {
        let cave1 = parse(INPUT1)?;
        let cave2 = parse(INPUT2)?;
        let cave3 = parse(INPUT3)?;
        assert_eq!(cave_paths(&cave1, PathSelection::Once), Some(10));
        assert_eq!(cave_paths(&cave2, PathSelection::Once), Some(19));
        assert_eq!(cave_paths(&cave3, PathSelection::Once), Some(226));
        Ok(())
    }
    #[test]
    fn part2() -> anyhow::Result<()> {
        let cave1 = parse(INPUT1)?;
        let cave2 = parse(INPUT2)?;
        let cave3 = parse(INPUT3)?;
        assert_eq!(cave_paths(&cave1, PathSelection::Twice), Some(36));
        assert_eq!(cave_paths(&cave2, PathSelection::Twice), Some(103));
        assert_eq!(cave_paths(&cave3, PathSelection::Twice), Some(3509));
        Ok(())
    }
}
