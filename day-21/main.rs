use itertools::Itertools;
use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{line_ending, u32},
    multi::separated_list0,
    sequence::preceded,
    Finish, Parser,
};
use once_cell::sync::OnceCell;
use std::{cell::RefCell, collections::HashMap, fs::read_to_string};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct Player {
    pos: u32,
    score: u32,
    id: usize,
}

impl Player {
    fn turn(mut self, roll: u32) -> Self {
        self.pos = (self.pos + roll - 1) % 10 + 1;
        self.score += self.pos;
        self
    }
}

fn parse(input: &str) -> anyhow::Result<Vec<Player>> {
    let positions = preceded(take_until(": ").and(tag(": ")), u32);
    let mut parser = separated_list0(line_ending, positions).map(|positions| {
        positions
            .into_iter()
            .enumerate()
            .map(|(id, pos)| Player { pos, id, score: 0 })
            .collect()
    });
    parser
        .parse(input)
        .finish()
        .map(|(_input, parsed)| parsed)
        .map_err(|e: nom::error::VerboseError<&str>| anyhow::anyhow!("parser error: {:?}", e))
}
struct Dice {
    win_score: u32,
    rolls_per_turn: usize,
    dice_sides: u32,
    players: Vec<Player>,
    cache: RefCell<HashMap<GameState, [u64; 2]>>,
}
#[derive(PartialEq, Eq, Hash)]
struct GameState((Player, Player));

impl Dice {
    fn new(win_score: u32, rolls_per_turn: usize, dice_sides: u32, players: Vec<Player>) -> Self {
        Self {
            win_score,
            rolls_per_turn,
            dice_sides,
            players,
            cache: RefCell::new(HashMap::new()),
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
    fn rolls_freq(&self) -> &HashMap<u32, usize> {
        static ROLLS: OnceCell<HashMap<u32, usize>> = OnceCell::new();
        ROLLS.get_or_init(|| {
            (1..=self.dice_sides)
                .cartesian_product(1..=self.dice_sides)
                .cartesian_product(1..=self.dice_sides)
                .map(|((a, b), c)| a + b + c)
                .counts()
        })
    }
    fn dirac_winner_counts(&self) -> [u64; 2] {
        self.universe(self.players[0], self.players[1])
    }

    fn universe(&self, p1: Player, p2: Player) -> [u64; 2] {
        if p2.score >= 21 {
            return [0, 1];
        }
        let state = GameState((p1, p2));
        if self.cache.borrow().contains_key(&state) {
            return self.cache.borrow()[&state];
        }

        let wins = self
            .rolls_freq()
            .iter()
            .fold([0, 0], |mut acc, (&roll, &freq)| {
                let [w0, w1] = self.universe(p2, p1.turn(roll));
                acc[0] += w1 * freq as u64;
                acc[1] += w0 * freq as u64;
                acc
            });
        self.cache.borrow_mut().insert(state, wins);
        wins
    }
}
fn main() -> anyhow::Result<()> {
    let input = read_to_string("day-21/input.txt")?;
    let players = parse(&input)?;
    let mut game = Dice::new(1000, 3, 100, players.clone());
    let part1 = game.determministic_rolls() * game.looser_score();
    println!("part1 result is {}", part1);

    let game = Dice::new(21, 3, 3, players);
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
        let game = Dice::new(21, 3, 3, players);
        let part2 = game.dirac_winner_counts();
        assert_eq!(part2[0], 444_356_092_776_315);
        Ok(())
    }
}
