fn parse(input: &str) -> Vec<Vec<u8>> {
    let mut lines = input.trim().lines().peekable();
    let num_bits = lines.peek().unwrap().len();
    //transpose input

    let mut res = vec![vec![]; num_bits];
    lines.for_each(|line| {
        for (i, b) in line.chars().enumerate() {
            let bit = b.to_digit(2).unwrap_or_default() as u8;
            res[i].push(bit);
        }
    });
    res
}

fn power_consumption(report: &[Vec<u8>]) -> u32 {
    let idxs: Vec<usize> = (0..report[0].len()).collect();
    let num_bits = report.len();
    let gamma = report.iter().enumerate().fold(0u32, |acc, (i, row)| {
        let mcb = most_common_bit(row, &idxs) as u32;
        acc + (mcb << (num_bits - 1 - i)) as u32
    });
    let eps = gamma ^ (u32::MAX >> (u32::BITS as usize - num_bits));
    gamma * eps
}
fn most_common_bit(v: &[u8], idxs: &[usize]) -> u8 {
    let mut ones: usize = 0;
    for i in idxs {
        ones += v[*i] as usize;
    }
    if ones >= idxs.len() - ones {
        1
    } else {
        0
    }
}

fn oxygen_co2(report: &[Vec<u8>]) -> u32 {
    let mut oxygen_idx: Vec<usize> = (0..report[0].len()).collect();
    let mut co2_idx: Vec<usize> = (0..report[0].len()).collect();
    for row in report.iter() {
        let mcb = most_common_bit(row, &oxygen_idx);
        if oxygen_idx.len() > 1 {
            oxygen_idx.retain(|&i| row[i] == mcb);
        }
        let lcb = most_common_bit(row, &co2_idx) ^ 1;
        if co2_idx.len() > 1 {
            co2_idx.retain(|&i| row[i] == lcb);
        }
    }
    let mut oxygen = 0u32;
    let mut co2 = 0u32;
    let num_bits = report.len();

    for (i, row) in report.iter().enumerate() {
        oxygen += (row[oxygen_idx[0]] as u32) << (num_bits - 1 - i);
        co2 += (row[co2_idx[0]] as u32) << (num_bits - 1 - i);
    }

    oxygen * co2
}
fn main() {
    let input = std::fs::read_to_string("day-03/input.txt").unwrap();
    let report = parse(&input);
    let part1 = power_consumption(&report);
    let part2 = oxygen_co2(&report);
    println!("part1 result is {}", part1);
    println!("part2 result is {}", part2);
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"
00100
11110
10110
10111
10101
01111
00111
11100
10000
11001
00010
01010
    "#;
    #[test]
    fn part1() {
        let report = parse(INPUT);
        assert_eq!(power_consumption(&report), 198);
    }
    #[test]
    fn part2() {
        let report = parse(INPUT);
        assert_eq!(oxygen_co2(&report), 230);
    }
}
