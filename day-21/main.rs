use itertools::Itertools;
use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{line_ending, u32},
    multi::separated_list0,
    sequence::preceded,
    Finish, Parser,
};
use std::{collections::HashMap, fs::read_to_string};

#[derive(Debug, Default, Clone, Copy)]
struct Player {
    pos: u32,
    score: u32,
}

impl Player {
    fn turn(self, roll: u32) -> Self {
        let new_pos = (self.pos + roll - 1) % 10 + 1;
        Self {
            pos: new_pos,
            score: self.score + new_pos,
        }
    }
}

fn parse(input: &str) -> anyhow::Result<Vec<Player>> {
    let player = preceded(take_until(": ").and(tag(": ")), u32).map(|pos| Player { pos, score: 0 });
    let mut parser = separated_list0(line_ending, player);
    parser
        .parse(input)
        .finish()
        .map(|(_input, parsed)| parsed)
        .map_err(|e: nom::error::VerboseError<&str>| anyhow::anyhow!("parser error: {:?}", e))
}
#[derive(Debug, Clone)]
struct Dice {
    win_score: u32,
    rolls_per_turn: usize,
    dice_sides: u32,
    players: Vec<Player>,
    rolls_freq: Option<HashMap<u32, usize>>,
}

impl Dice {
    fn new(win_score: u32, rolls_per_turn: usize, dice_sides: u32, players: Vec<Player>) -> Self {
        Self {
            win_score,
            rolls_per_turn,
            dice_sides,
            players,
            rolls_freq: None,
        }
    }
    fn determministic_rolls(&mut self) -> usize {
        let players = (0..self.players.len()).cycle();
        let rolls = (1..=self.dice_sides)
            .into_iter()
            .cycle()
            .chunks(self.rolls_per_turn);
        let roll_count = players
            .zip(&rolls)
            .map(|(i, roll)| {
                self.players[i] = self.players[i].turn(roll.sum());
                self.players[i].score
            })
            .take_while(|score| score < &self.win_score)
            .count()
            + 1;
        roll_count * self.rolls_per_turn
    }
    fn looser_score(&self) -> usize {
        self.players
            .iter()
            .map(|player| player.score)
            .min()
            .unwrap() as usize
    }
    fn dirac_winner_counts(&mut self) -> [u64; 2] {
        self.rolls_freq = Some(
            (1..=self.dice_sides)
                .cartesian_product(1..=self.dice_sides)
                .cartesian_product(1..=self.dice_sides)
                .map(|((a, b), c)| a + b + c)
                .counts(),
        );
        self.universe(self.players[0], self.players[1])
    }

    fn universe(&self, p1: Player, p2: Player) -> [u64; 2] {
        if p1.score >= 21 {
            return [1, 0];
        }
        if p2.score >= 21 {
            return [0, 1];
        }
        let mut wins = [0, 0];
        for (&roll, &freq) in self.rolls_freq.as_ref().unwrap() {
            let [w0, w1] = self.universe(p2, p1.turn(roll));
            wins[0] += w1 * freq as u64;
            wins[1] += w0 * freq as u64;
        }
        wins
    }
}
fn main() -> anyhow::Result<()> {
    let input = read_to_string("day-21/input.txt")?;
    let players = parse(&input)?;
    let mut game = Dice::new(1000, 3, 100, players.clone());
    let part1 = game.determministic_rolls() * game.looser_score();
    println!("part1 result is {}", part1);

    let mut game = Dice::new(21, 3, 3, players);
    let part2 = game.dirac_winner_counts().into_iter().max().unwrap();
    println!("part2 result is {}", part2);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const INPUT: &str = r#"Player 1 starting position: 4
Player 2 starting position: 8
"#;

    #[test]
    fn part1() -> anyhow::Result<()> {
        let players = parse(INPUT.trim())?;
        let mut game = Dice::new(1000, 3, 100, players);
        let part1 = game.determministic_rolls() * game.looser_score();
        assert_eq!(part1, 739785);
        Ok(())
    }
    #[test]
    fn part2() -> anyhow::Result<()> {
        let players = parse(INPUT)?;
        let mut game = Dice::new(21, 3, 3, players);
        let part2 = game.dirac_winner_counts();
        assert_eq!(part2[0], 444_356_092_776_315);
        Ok(())
    }
}
