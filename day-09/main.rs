use std::cmp::Reverse;
use std::collections::HashSet;
use std::fs::read_to_string;

#[derive(Debug, Clone)]
struct ParsedInput {
    height_map: Vec<Vec<u32>>,
}
fn parse(input: &str) -> anyhow::Result<ParsedInput> {
    let height_map = input
        .lines()
        .map(|line| line.chars().map(|c| c.to_digit(10).unwrap()).collect())
        .collect();
    Ok(ParsedInput { height_map })
}
fn main() -> anyhow::Result<()> {
    let input = read_to_string("day-09/input.txt").unwrap();
    let parsed = parse(&input)?;

    let lowest_points = lowest_points(&parsed);
    let part1 = risk_level(&parsed.height_map, &lowest_points);
    println!("part1 result is {}", part1);
    let part2 = basin_sizes_mul(&parsed.height_map, &lowest_points);
    println!("part2 result is {}", part2);
    Ok(())
}

fn risk_level(height_map: &[Vec<u32>], lowest_points: &[(usize, usize)]) -> u32 {
    lowest_points
        .iter()
        .fold(0, |acc, (x, y)| acc + height_map[*x][*y] + 1)
}
fn basin_fill(hm: &[Vec<u32>], point: (usize, usize), basin: &mut HashSet<(usize, usize)>) {
    basin.insert(point);
    let (i, j) = point;
    let left = i.checked_sub(1).map(|i| (i, j));
    let up = j.checked_sub(1).map(|j| (i, j));
    let right = (i + 1 < hm.len()).then(|| (i + 1, j));
    let down = (j + 1 < hm[0].len()).then(|| (i, j + 1));
    let mut adj_points: Vec<_> = [left, right, up, down]
        .iter()
        .copied()
        .filter(|h| {
            if h.is_none() {
                return false;
            }
            let (x, y) = h.unwrap();

            hm[x][y] != 9 && hm[x][y] > hm[i][j] && basin.get(&(x, y)).is_none()
        })
        .map(Option::unwrap)
        .collect();
    for adj in adj_points.drain(..) {
        basin_fill(hm, adj, basin);
    }
}
fn basin_sizes_mul(hm: &[Vec<u32>], lowest_points: &[(usize, usize)]) -> u32 {
    let mut basins = Vec::with_capacity(lowest_points.len());
    for lp in lowest_points {
        basins.push(HashSet::new());
        basin_fill(hm, *lp, basins.last_mut().unwrap());
    }
    basins.sort_unstable_by_key(|basin| Reverse(basin.len()));
    basins.iter().take(3).fold(1, |acc, v| acc * v.len() as u32)
}
fn lowest_points(input: &ParsedInput) -> Vec<(usize, usize)> {
    let hm = &input.height_map;
    let rows = hm.len();
    let cols = hm[0].len();
    let mut result = vec![];
    for i in 0..rows {
        for j in 0..cols {
            let left = i.checked_sub(1).map(|i| &hm[i][j]);
            let up = j.checked_sub(1).map(|j| &hm[i][j]);
            let right = hm.get(i + 1).and_then(|row| row.get(j));
            let down = hm.get(i).and_then(|row| row.get(j + 1));
            let is_min = [left, right, up, down]
                .iter()
                .filter(|h| h.is_some())
                .copied()
                .all(|h| *h.unwrap() > hm[i][j]);

            if is_min {
                result.push((i, j));
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    const INPUT: &str = r#"2199943210
3987894921
9856789892
8767896789
9899965678
"#;
    #[test]
    fn part1() -> anyhow::Result<()> {
        let parsed = parse(INPUT)?;
        let lowest_points = lowest_points(&parsed);
        assert_eq!(risk_level(&parsed.height_map, &lowest_points), 15);
        Ok(())
    }
    #[test]
    fn part2() -> anyhow::Result<()> {
        let parsed = parse(INPUT)?;
        let lowest_points = lowest_points(&parsed);
        assert_eq!(basin_sizes_mul(&parsed.height_map, &lowest_points), 1134);
        Ok(())
    }
}
