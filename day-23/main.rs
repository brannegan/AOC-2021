use std::cmp::{max, min};
use std::ops::Not;

use anyhow::Ok;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Amphipod {
    Amber,
    Bronze,
    Copper,
    Desert,
}
impl Amphipod {
    fn home(&self) -> usize {
        match self {
            Amphipod::Amber => 0,
            Amphipod::Bronze => 1,
            Amphipod::Copper => 2,
            Amphipod::Desert => 3,
        }
    }
    fn energy(&self) -> usize {
        match self {
            Amphipod::Amber => 1,
            Amphipod::Bronze => 10,
            Amphipod::Copper => 100,
            Amphipod::Desert => 1000,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Tile {
    Empty,
    Entrance(usize),
    Occupied(Amphipod),
}
impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "."),
            Self::Entrance(_) => write!(f, "_"),
            Self::Occupied(amphi) => write!(
                f,
                "{}",
                match amphi {
                    Amphipod::Amber => 'A',
                    Amphipod::Bronze => 'B',
                    Amphipod::Copper => 'C',
                    Amphipod::Desert => 'D',
                }
            ),
        }
    }
}
#[derive(Clone, Copy, Debug)]
struct Burrow<const N: usize> {
    hallway: [Tile; 11],
    rooms: [[Tile; N]; 4],
}

impl<const N: usize> std::fmt::Display for Burrow<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:#<13}", "")?;
        write!(f, "#")?;
        for t in self.hallway {
            write!(f, "{}", t)?;
        }
        writeln!(f, "#")?;
        for c in 0..self.rooms[0].len() {
            write!(f, "###")?;
            for r in 0..self.rooms.len() {
                write!(f, "{}#", self.rooms[r][c])?;
            }
            writeln!(f, "##")?;
        }
        writeln!(f, "{:^13}", "#######")
    }
}

impl<const N: usize> Burrow<N> {
    fn new(rooms: [[Tile; N]; 4]) -> Self {
        let mut hallway = [Tile::Empty; 11];
        for i in 0..rooms.len() {
            hallway[2 + 2 * i] = Tile::Entrance(i);
        }

        Self { hallway, rooms }
    }
    fn required_state(&self) -> bool {
        self.rooms
            == [
                [Tile::Occupied(Amphipod::Amber); N],
                [Tile::Occupied(Amphipod::Bronze); N],
                [Tile::Occupied(Amphipod::Copper); N],
                [Tile::Occupied(Amphipod::Desert); N],
            ]
    }
    fn min_cost(&self) -> Option<usize> {
        if self.required_state() {
            return Some(0);
        }
        let to_room_iter = self
            .hallway
            .iter()
            .enumerate()
            .filter_map(|(pos, tile)| match tile {
                Tile::Empty => None,
                Tile::Entrance(_) => None,
                Tile::Occupied(amphi) => self.try_move_to_room(pos, *amphi),
            });
        let to_hallway_iter = self
            .rooms
            .iter()
            .enumerate()
            .filter_map(|(room_num, room)| {
                room.iter()
                    .all(|tile| match tile {
                        Tile::Empty => true,
                        Tile::Occupied(amphi) if room_num == amphi.home() => true,
                        _ => false,
                    })
                    .not()
                    .then(|| {
                        room.iter()
                            .position(|tile| *tile != Tile::Empty)
                            .and_then(|room_pos| self.try_move_to_hall(room_num, room_pos))
                    })
                    .flatten()
            })
            .flatten();

        to_room_iter
            .chain(to_hallway_iter)
            .filter_map(|(cost, next_state)| next_state.min_cost().map(|min_cost| cost + min_cost))
            .min()
    }
    fn try_move_to_hall(
        &self,
        room_num: usize,
        room_pos: usize,
    ) -> Option<Vec<(usize, Burrow<N>)>> {
        let amphi = if let Tile::Occupied(amphi) = self.rooms[room_num][room_pos] {
            amphi
        } else {
            return None;
        };
        let room_entrance = self
            .hallway
            .iter()
            .position(|tile| *tile == Tile::Entrance(room_num))
            .unwrap();
        let left_path = (0..room_entrance)
            .rev()
            .filter(|&i| !matches!(self.hallway[i], Tile::Entrance(_)))
            .take_while(|&i| self.hallway[i] == Tile::Empty);
        let right_path = (room_entrance + 1..self.hallway.len())
            .filter(|&i| !matches!(self.hallway[i], Tile::Entrance(_)))
            .take_while(|&i| self.hallway[i] == Tile::Empty);
        Some(
            left_path
                .chain(right_path)
                .map(|i| {
                    let hall_path = min(room_entrance, i)..max(room_entrance, i);
                    let energy_cost = (hall_path.len() + room_pos + 1) * amphi.energy();
                    let mut new_burrow = *self;
                    std::mem::swap(
                        &mut new_burrow.hallway[i],
                        &mut new_burrow.rooms[room_num][room_pos],
                    );
                    (energy_cost, new_burrow)
                })
                .collect(),
        )
    }
    fn try_move_to_room(&self, hall_pos: usize, amphi: Amphipod) -> Option<(usize, Burrow<N>)> {
        let home = amphi.home();
        let home_entrance = self
            .hallway
            .iter()
            .position(|tile| *tile == Tile::Entrance(home))
            .unwrap();
        let mut hall_path = min(home_entrance, hall_pos)..max(home_entrance, hall_pos);
        let hall_path_len = hall_path.len();
        if hall_path.any(|i| i != hall_pos && matches!(self.hallway[i], Tile::Occupied(_))) {
            return None; //Path is blocked
        }
        let room_pos = self.rooms[home]
            .iter()
            .all(|tile| match tile {
                Tile::Empty => true,
                Tile::Occupied(other) if home == other.home() => true,
                _ => false,
            })
            .then(|| {
                self.rooms[home]
                    .iter()
                    .position(|tile| *tile == Tile::Empty)
            })
            .flatten()?;
        let energy_cost = (hall_path_len + room_pos + 1) * amphi.energy();
        let mut new_burrow = *self;
        std::mem::swap(
            &mut new_burrow.hallway[hall_pos],
            &mut new_burrow.rooms[amphi.home()][room_pos],
        );

        Some((energy_cost, new_burrow))
    }
}

fn main() -> anyhow::Result<()> {
    let rooms1 = [
        [
            Tile::Occupied(Amphipod::Desert),
            Tile::Occupied(Amphipod::Copper),
        ],
        [
            Tile::Occupied(Amphipod::Copper),
            Tile::Occupied(Amphipod::Amber),
        ],
        [
            Tile::Occupied(Amphipod::Desert),
            Tile::Occupied(Amphipod::Amber),
        ],
        [
            Tile::Occupied(Amphipod::Bronze),
            Tile::Occupied(Amphipod::Bronze),
        ],
    ];
    let rooms2 = [
        [
            Tile::Occupied(Amphipod::Desert),
            Tile::Occupied(Amphipod::Desert),
            Tile::Occupied(Amphipod::Desert),
            Tile::Occupied(Amphipod::Copper),
        ],
        [
            Tile::Occupied(Amphipod::Copper),
            Tile::Occupied(Amphipod::Copper),
            Tile::Occupied(Amphipod::Bronze),
            Tile::Occupied(Amphipod::Amber),
        ],
        [
            Tile::Occupied(Amphipod::Desert),
            Tile::Occupied(Amphipod::Bronze),
            Tile::Occupied(Amphipod::Amber),
            Tile::Occupied(Amphipod::Amber),
        ],
        [
            Tile::Occupied(Amphipod::Bronze),
            Tile::Occupied(Amphipod::Amber),
            Tile::Occupied(Amphipod::Copper),
            Tile::Occupied(Amphipod::Bronze),
        ],
    ];
    let burrow1 = Burrow::new(rooms1);
    let part1 = burrow1.min_cost();
    println!("{:?}", part1);
    let burrow2 = Burrow::new(rooms2);
    let part2 = burrow2.min_cost();
    println!("{:?}", part2);
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn part1() -> anyhow::Result<()> {
        #[rustfmt::skip]
        let rooms = [
            [Tile::Occupied(Amphipod::Bronze), Tile::Occupied(Amphipod::Amber)],
            [Tile::Occupied(Amphipod::Copper), Tile::Occupied(Amphipod::Desert)],
            [Tile::Occupied(Amphipod::Bronze), Tile::Occupied(Amphipod::Copper)],
            [Tile::Occupied(Amphipod::Desert), Tile::Occupied(Amphipod::Amber)],
        ];
        let burrow = Burrow::new(rooms);
        let min = burrow.min_cost();
        assert_eq!(min, Some(12521));
        Ok(())
    }
    #[test]
    fn part2() -> anyhow::Result<()> {
        let rooms = [
            [
                Tile::Occupied(Amphipod::Bronze),
                Tile::Occupied(Amphipod::Desert),
                Tile::Occupied(Amphipod::Desert),
                Tile::Occupied(Amphipod::Amber),
            ],
            [
                Tile::Occupied(Amphipod::Copper),
                Tile::Occupied(Amphipod::Copper),
                Tile::Occupied(Amphipod::Bronze),
                Tile::Occupied(Amphipod::Desert),
            ],
            [
                Tile::Occupied(Amphipod::Bronze),
                Tile::Occupied(Amphipod::Bronze),
                Tile::Occupied(Amphipod::Amber),
                Tile::Occupied(Amphipod::Copper),
            ],
            [
                Tile::Occupied(Amphipod::Desert),
                Tile::Occupied(Amphipod::Amber),
                Tile::Occupied(Amphipod::Copper),
                Tile::Occupied(Amphipod::Amber),
            ],
        ];
        let burrow = Burrow::new(rooms);
        let min = burrow.min_cost();
        assert_eq!(min, Some(44169));
        Ok(())
    }
}
