use nom::combinator::{flat_map, map};
use nom::multi::many_till;
use nom::sequence::{preceded, tuple};
use nom::{bits::complete::tag, bits::complete::take};
use nom::{IResult, Parser};
use std::{fs::read_to_string, u8};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Packet {
    version: u8,
    kind: PacketKind,
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum PacketKind {
    Literal(u64),
    Operator {
        length_type: LengthType,
        packets: Vec<Packet>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LengthType {
    Bits(u32),
    Subpackets(u32),
}
type ParseResult<'a, T> = IResult<(&'a [u8], usize), T>;

fn remaining_len(input: (&[u8], usize)) -> usize {
    input.0.len() * 8 - input.1
}

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
fn parse_operator(input: (&[u8], usize)) -> ParseResult<PacketKind> {
    let (input, length_type_id) = take(1usize)(input)?;
    let mut bits = take(15usize).map(LengthType::Bits);
    let mut packets = take(11usize).map(LengthType::Subpackets);
    let (mut input, length_type) = match length_type_id {
        0 => bits.parse(input)?,
        1 => packets.parse(input)?,
        _ => unreachable!(),
    };
    let mut packets = vec![];
    match length_type {
        LengthType::Bits(len) => {
            let start_len = remaining_len(input);
            let mut cur_len = start_len;
            while start_len - cur_len < len as usize {
                let (inner_input, packet) = Packet::parse_from_offset(input)?;
                input = inner_input;
                cur_len = remaining_len(input);
                packets.push(packet);
            }
        }
        LengthType::Subpackets(num) => {
            while packets.len() < num as usize {
                let (inner_input, packet) = Packet::parse_from_offset(input)?;
                input = inner_input;
                packets.push(packet);
            }
        }
    }
    Ok((
        input,
        PacketKind::Operator {
            length_type,
            packets,
        },
    ))
}
impl Packet {
    fn parse(input: &[u8]) -> ParseResult<Packet> {
        Self::parse_from_offset((input, 0))
    }
    fn parse_from_offset(input: (&[u8], usize)) -> ParseResult<Self> {
        let version = take(3usize);
        let type_id = take(3usize);
        let kind = flat_map(type_id, |type_id| match type_id {
            4 => parse_literal,
            _ => parse_operator,
        });
        let mut parser = map(tuple((version, kind)), |(version, kind)| Packet {
            version,
            kind,
        });
        parser.parse(input)
    }
    fn version_sum(&self) -> u32 {
        match &self.kind {
            PacketKind::Literal(_) => self.version as u32,
            PacketKind::Operator { packets, .. } => {
                packets.iter().map(|p| p.version_sum()).sum::<u32>() + self.version as u32
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let input = read_to_string("day-16/input.txt")?;
    let input = hex::decode(input.trim())?;
    let (_, packet) = Packet::parse(&input).map_err(|_| anyhow::anyhow!("parse packet error"))?;
    println!("part1 result is {}", packet.version_sum());
    //println!("part2 result is {}", part2);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    const PACKET: [u8; 3] = hex!("D2FE28");
    const OPERATOR_PACKET1: [u8; 7] = hex!("38006F45291200");
    const OPERATOR_PACKET2: [u8; 7] = hex!("EE00D40C823060");
    #[test]
    fn part1_1() -> anyhow::Result<()> {
        let (_, packet) = Packet::parse(&PACKET)?;
        assert_eq!(
            packet,
            Packet {
                version: 6,
                kind: PacketKind::Literal(2021)
            }
        );
        Ok(())
    }
    #[test]
    fn part1_2() -> anyhow::Result<()> {
        let (_, op_packet) = Packet::parse(&OPERATOR_PACKET1)?;
        assert_eq!(
            op_packet.kind,
            PacketKind::Operator {
                length_type: LengthType::Bits(27),
                packets: vec![
                    Packet {
                        version: 6,
                        kind: PacketKind::Literal(10)
                    },
                    Packet {
                        version: 2,
                        kind: PacketKind::Literal(20)
                    },
                ],
            }
        );
        Ok(())
    }
    #[test]
    fn part1_3() -> anyhow::Result<()> {
        let (_, op_packet) = Packet::parse(&OPERATOR_PACKET2)?;
        assert_eq!(
            op_packet.kind,
            PacketKind::Operator {
                length_type: LengthType::Subpackets(3),
                packets: vec![
                    Packet {
                        version: 2,
                        kind: PacketKind::Literal(1)
                    },
                    Packet {
                        version: 4,
                        kind: PacketKind::Literal(2)
                    },
                    Packet {
                        version: 1,
                        kind: PacketKind::Literal(3)
                    },
                ],
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
    //#[test]
    //fn part2() -> anyhow::Result<()> {
    //    let parsed = parse(INPUT)?;
    //    let tiled_map = tiled(parsed, 5);
    //    let tiled_map = RiskMap::new(tiled_map);
    //    assert_eq!(path_risk_level(&tiled_map), 315);
    //    Ok(())
    //}
}
