use anyhow::Ok;
use nom::{
    character::complete::{char, line_ending, space0, space1, u32},
    combinator::map,
    multi::separated_list1,
    Finish, Parser,
};
use std::{fs::read_to_string, ops::Not};

const BOARD_SIZE: usize = 5;

#[derive(Debug, Clone)]
struct Board {
    cells: [[(u32, bool); BOARD_SIZE]; BOARD_SIZE],
}
impl Board {
    fn is_winner(&mut self, row: usize, col: usize) -> bool {
        let mut col_win = true;
        for r in 0..BOARD_SIZE {
            col_win = col_win && self.cells[r][col].1;
        }
        if col_win {
            return true;
        }
        let mut row_win = true;
        for c in 0..BOARD_SIZE {
            row_win = row_win && self.cells[row][c].1;
        }
        if row_win {
            return true;
        }
        false
    }
    fn sum_unmarked(&self) -> u32 {
        self.cells
            .iter()
            .copied()
            .flatten()
            .filter_map(|(num, marked)| marked.not().then(|| num))
            .sum::<u32>()
    }
}

#[derive(Debug)]
struct Draw {
    numbers: Vec<u32>,
}
#[derive(Debug)]
struct ParsedInput {
    draw: Draw,
    boards: Vec<Board>,
}

fn parse(input: &str) -> anyhow::Result<ParsedInput> {
    let draw = separated_list1(char(','), u32).map(|numbers| Draw { numbers });
    let gap = |i| line_ending.and(line_ending).parse(i);
    let row = space0
        .and(separated_list1(space1, u32.map(|num| (num, false))))
        .map(|(_space0, row)| row.try_into().expect("Wrong board width"));
    let board = separated_list1(line_ending, row).map(|cells| Board {
        cells: cells.try_into().expect("Wrong board height"),
    });
    let boards = separated_list1(gap, board);
    let mut parser = map(draw.and(gap).and(boards), |((draw, _), boards)| {
        ParsedInput { draw, boards }
    });

    parser
        .parse(input)
        .finish()
        .map(|(_input, parsed)| parsed)
        .map_err(|_: nom::error::Error<&str>| anyhow::anyhow!("parser error"))
}
fn winner_check(board: &mut Board, num: u32) -> bool {
    for c in 0..BOARD_SIZE {
        for r in 0..BOARD_SIZE {
            let cell = &mut board.cells[r][c];
            if cell.0 == num {
                cell.1 = true;
                if board.is_winner(r, c) {
                    return true;
                }
            }
        }
    }
    false
}

fn first_winner_score(input: &ParsedInput) -> Option<u32> {
    let mut boards = input.boards.clone();
    for num in &input.draw.numbers {
        for board in &mut boards {
            if winner_check(board, *num) {
                return Some(board.sum_unmarked() * num);
            }
        }
    }
    None
}
fn last_winner_score(input: &ParsedInput) -> Option<u32> {
    let mut boards = input.boards.clone();
    let mut win_board_idxs = vec![];
    for num in &input.draw.numbers {
        let boards_left = boards.len();
        for (idx, board) in boards.iter_mut().enumerate() {
            if winner_check(board, *num) {
                win_board_idxs.push(idx);
                if boards_left == 1 {
                    return Some(board.sum_unmarked() * num);
                }
            }
        }
        win_board_idxs.drain(..).rev().for_each(|idx| {
            let _ = boards.swap_remove(idx);
        });
    }
    None
}
fn main() -> anyhow::Result<()> {
    let input = read_to_string("day-04/input.txt")?;
    let parsed = parse(&input)?;
    let part1 = first_winner_score(&parsed).expect("No one won yet");
    println!("part1 result is {}", part1);
    let part2 = last_winner_score(&parsed).expect("More boards left");
    println!("part2 result is {}", part2);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r#"7,4,9,5,11,17,23,2,0,14,21,24,10,16,13,6,15,25,12,22,18,20,8,19,3,26,1

22 13 17 11  0
 8  2 23  4 24
21  9 14 16  7
 6 10  3 18  5
 1 12 20 15 19

 3 15  0  2 22
 9 18 13 17  5
19  8  7 25 23
20 11 10 24  4
14 21 16 12  6

14 21 17 24  4
10 16 15  9 19
18  8 23 26 20
22 11 13  6  5
 2  0 12  3  7
"#;
    #[test]
    fn part1() -> anyhow::Result<()> {
        let parsed = parse(INPUT)?;
        assert_eq!(first_winner_score(&parsed), Some(4512));
        Ok(())
    }
    #[test]
    fn part2() -> anyhow::Result<()> {
        let parsed = parse(INPUT)?;
        assert_eq!(last_winner_score(&parsed), Some(1924));
        Ok(())
    }
}
