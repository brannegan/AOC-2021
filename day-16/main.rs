use nom::combinator::map;
use nom::multi::{length_count, many_till};
use nom::sequence::preceded;
use nom::{bits::complete::tag, bits::complete::take};
use nom::{IResult, Parser};
use std::hint::unreachable_unchecked;
use std::{fs::read_to_string, u8};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Packet {
    version: u8,
    kind: PacketKind,
    data: Option<Vec<Packet>>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum PacketKind {
    Sum,
    Prod,
    Min,
    Max,
    Literal(u64),
    Gt,
    Lt,
    Eq,
}

type ParseResult<'a, T> = IResult<(&'a [u8], usize), T>;

fn parse_literal(input: (&[u8], usize)) -> ParseResult<PacketKind> {
    let group = preceded(tag(0b1, 1usize), take(4usize));
    let group_last = preceded(tag(0b0, 1usize), take(4usize));
    let groups = many_till(group, group_last);
    let mut parser = map(groups, |(groups, last): (Vec<u8>, u8)| {
        let literal = groups
            .iter()
            .copied()
            .rev()
            .enumerate()
            .fold(last as u64, |acc, (i, n)| {
                let shifted_n = (n as u64) << ((i + 1) * 4);
                acc + shifted_n
            });
        PacketKind::Literal(literal)
    });
    parser.parse(input)
}
fn parse_subpackets(input: (&[u8], usize)) -> ParseResult<Vec<Packet>> {
    let (input, length_type_id) = take(1usize)(input)?;
    match length_type_id {
        0 => {
            fn remaining_len(input: (&[u8], usize)) -> usize {
                input.0.len() * 8 - input.1
            }
            let (mut input, len) = take(15usize)(input)?;
            let mut packets = vec![];
            let mut cur_len = remaining_len(input);
            let start_len = cur_len;
            while start_len - cur_len < len {
                let (inner_input, packet) = Packet::parse_from_offset(input)?;
                input = inner_input;
                cur_len = remaining_len(input);
                packets.push(packet);
            }
            Ok((input, packets))
        }
        1 => {
            let num = take::<_, u16, _, _>(11usize);
            let mut parser = length_count(num, Packet::parse_from_offset);
            parser.parse(input)
        }
        // SAFETY: match on 1 bit value
        _ => unsafe { unreachable_unchecked() },
    }
}
impl Packet {
    fn parse(input: &[u8]) -> ParseResult<Packet> {
        Self::parse_from_offset((input, 0))
    }
    fn parse_from_offset(input: (&[u8], usize)) -> ParseResult<Self> {
        let (input, version) = take(3usize)(input)?;
        let (mut input, type_id) = take(3usize)(input)?;
        let kind = match type_id {
            0 => PacketKind::Sum,
            1 => PacketKind::Prod,
            2 => PacketKind::Min,
            3 => PacketKind::Max,
            4 => {
                let (i, literal) = parse_literal(input)?;
                input = i;
                literal
            }
            5 => PacketKind::Gt,
            6 => PacketKind::Lt,
            7 => PacketKind::Eq,
            _ => unimplemented!(),
        };
        let data = match kind {
            PacketKind::Literal(_) => None,
            _ => {
                let (i, packets) = parse_subpackets(input)?;
                input = i;
                Some(packets)
            }
        };
        Ok((
            input,
            Packet {
                version,
                kind,
                data,
            },
        ))
    }
    fn version_sum(&self) -> u64 {
        match &self.kind {
            PacketKind::Literal(_) => self.version as u64,
            _ => {
                self.data
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|p| p.version_sum())
                    .sum::<u64>()
                    + self.version as u64
            }
        }
    }
    fn eval(&self) -> u64 {
        let empty = vec![];
        let packets = self.data.as_ref().unwrap_or(&empty);
        match &self.kind {
            PacketKind::Sum => packets.iter().map(|p| p.eval()).sum(),
            PacketKind::Prod => packets.iter().map(|p| p.eval()).product(),
            PacketKind::Min => packets.iter().map(|p| p.eval()).min().unwrap(),
            PacketKind::Max => packets.iter().map(|p| p.eval()).max().unwrap(),
            PacketKind::Literal(n) => *n,
            PacketKind::Gt => (packets[0].eval() > packets[1].eval()) as u64,
            PacketKind::Lt => (packets[0].eval() < packets[1].eval()) as u64,
            PacketKind::Eq => (packets[0].eval() == packets[1].eval()) as u64,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let input = read_to_string("day-16/input.txt")?;
    let input = hex::decode(input.trim())?;
    let (_, packet) = Packet::parse(&input).map_err(|_| anyhow::anyhow!("parse packet error"))?;
    println!("part1 result is {}", packet.version_sum());
    println!("part2 result is {}", packet.eval());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn part1_1() -> anyhow::Result<()> {
        const PACKET: [u8; 3] = hex!("D2FE28");
        let (_, packet) = Packet::parse(&PACKET)?;
        assert_eq!(
            packet,
            Packet {
                version: 6,
                kind: PacketKind::Literal(2021),
                data: None,
            }
        );
        Ok(())
    }
    #[test]
    fn part1_2() -> anyhow::Result<()> {
        let (_, op_packet) = Packet::parse(&hex!("38006F45291200"))?;
        assert_eq!(
            op_packet,
            Packet {
                version: 1,
                kind: PacketKind::Lt,
                data: Some(vec![
                    Packet {
                        version: 6,
                        kind: PacketKind::Literal(10),
                        data: None,
                    },
                    Packet {
                        version: 2,
                        kind: PacketKind::Literal(20),
                        data: None,
                    },
                ]),
            }
        );
        Ok(())
    }
    #[test]
    fn part1_3() -> anyhow::Result<()> {
        let (_, op_packet) = Packet::parse(&hex!("EE00D40C823060"))?;
        assert_eq!(
            op_packet,
            Packet {
                version: 7,
                kind: PacketKind::Max,
                data: Some(vec![
                    Packet {
                        version: 2,
                        kind: PacketKind::Literal(1),
                        data: None,
                    },
                    Packet {
                        version: 4,
                        kind: PacketKind::Literal(2),
                        data: None,
                    },
                    Packet {
                        version: 1,
                        kind: PacketKind::Literal(3),
                        data: None,
                    },
                ],)
            }
        );

        Ok(())
    }
    #[test]
    fn part1_4() -> anyhow::Result<()> {
        let (_, packet) = Packet::parse(&hex!("8A004A801A8002F478"))?;
        assert_eq!(packet.version_sum(), 16);
        Ok(())
    }
    #[test]
    fn part1_5() -> anyhow::Result<()> {
        let (_, packet) = Packet::parse(&hex!("620080001611562C8802118E34"))?;
        assert_eq!(packet.version_sum(), 12);
        Ok(())
    }
    #[test]
    fn part1_6() -> anyhow::Result<()> {
        let (_, packet) = Packet::parse(&hex!("C0015000016115A2E0802F182340"))?;
        assert_eq!(packet.version_sum(), 23);
        Ok(())
    }
    #[test]
    fn part1_7() -> anyhow::Result<()> {
        let (_, packet) = Packet::parse(&hex!("A0016C880162017C3686B18A3D4780"))?;
        assert_eq!(packet.version_sum(), 31);
        Ok(())
    }

    #[test]
    fn part2_1() -> anyhow::Result<()> {
        let (_, packet) = Packet::parse(&hex!("C200B40A82"))?;
        assert_eq!(packet.eval(), 3);
        Ok(())
    }
    #[test]
    fn part2_2() -> anyhow::Result<()> {
        let (_, packet) = Packet::parse(&hex!("04005AC33890"))?;
        assert_eq!(packet.eval(), 54);
        Ok(())
    }
    #[test]
    fn part2_3() -> anyhow::Result<()> {
        let (_, packet) = Packet::parse(&hex!("880086C3E88112"))?;
        assert_eq!(packet.eval(), 7);
        Ok(())
    }
    #[test]
    fn part2_4() -> anyhow::Result<()> {
        let (_, packet) = Packet::parse(&hex!("CE00C43D881120"))?;
        assert_eq!(packet.eval(), 9);
        Ok(())
    }
    #[test]
    fn part2_5() -> anyhow::Result<()> {
        let (_, packet) = Packet::parse(&hex!("D8005AC2A8F0"))?;
        assert_eq!(packet.eval(), 1);
        Ok(())
    }
    #[test]
    fn part2_6() -> anyhow::Result<()> {
        let (_, packet) = Packet::parse(&hex!("F600BC2D8F"))?;
        assert_eq!(packet.eval(), 0);
        Ok(())
    }
    #[test]
    fn part2_7() -> anyhow::Result<()> {
        let (_, packet) = Packet::parse(&hex!("9C005AC2F8F0"))?;
        assert_eq!(packet.eval(), 0);
        Ok(())
    }
    #[test]
    fn part2_8() -> anyhow::Result<()> {
        let (_, packet) = Packet::parse(&hex!("9C0141080250320F1802104A08"))?;
        assert_eq!(packet.eval(), 1);
        Ok(())
    }
}
