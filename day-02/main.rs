use std::fs::read_to_string;

//use itertools::Itertools;
enum Command {
    Forward(u8),
    Up(u8),
    Down(u8),
}

fn parse(input: &str) -> Vec<Command> {
    input
        .trim()
        .lines()
        .map(|line| match line.split_once(' ').unwrap() {
            ("forward", x) => Command::Forward(x.parse().unwrap_or_default()),
            ("up", x) => Command::Up(x.parse().unwrap_or_default()),
            ("down", x) => Command::Down(x.parse().unwrap_or_default()),
            _ => unreachable!(),
        })
        .collect()
}
fn distance(commands: &[Command]) -> u32 {
    let (pos, depth) = commands
        .iter()
        .fold((0u32, 0u32), |(pos, depth), command| match command {
            Command::Forward(units) => (pos + *units as u32, depth),
            Command::Up(units) => (pos, depth - *units as u32),
            Command::Down(units) => (pos, depth + *units as u32),
        });
    pos * depth
}
fn distance_with_aim(commands: &[Command]) -> u32 {
    let (pos, depth, _) =
        commands.iter().fold(
            (0u32, 0u32, 0u32),
            |(pos, depth, aim), command| match *command {
                Command::Forward(units) => (pos + units as u32, depth + aim * units as u32, aim),
                Command::Up(units) => (pos, depth, aim - units as u32),
                Command::Down(units) => (pos, depth, aim + units as u32),
            },
        );
    pos * depth
}
fn main() {
    let input = read_to_string("day-02/input.txt").unwrap();
    let commands = parse(&input);
    let part1 = distance(&commands);
    let part2 = distance_with_aim(&commands);
    println!("part1 result is {}", part1);
    println!("part2 result is {}", part2);
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"
forward 5
down 5
forward 8
up 3
down 8
forward 2"#;
    #[test]
    fn part1() {
        let commands = parse(INPUT);
        assert_eq!(distance(&commands), 150);
    }
    #[test]
    fn part2() {
        let commands = parse(INPUT);
        assert_eq!(distance_with_aim(&commands), 900);
    }
}
