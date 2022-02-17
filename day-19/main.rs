use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::complete::{char, i32, line_ending},
    combinator::map,
    multi::separated_list1,
    sequence::{delimited, tuple},
    Finish, Parser,
};
use std::fs::read_to_string;
#[rustfmt::skip]
const ORIENTATION: [[[i32; 3]; 3];24] = 
[
    [
        [ 1,  0,  0], 
        [ 0,  1,  0], 
        [ 0,  0,  1]
    ],
    [
        [ 0,  1,  0], 
        [-1,  0,  0], 
        [ 0,  0,  1]
    ],
    [
        [-1,  0,  0], 
        [ 0, -1,  0], 
        [ 0,  0,  1]
    ],
    [
        [ 0, -1,  0], 
        [ 1,  0,  0], 
        [ 0,  0,  1]
    ],
    [
        [-1,  0,  0], 
        [ 0,  1,  0], 
        [ 0,  0, -1]
    ],
    [
        [ 0,  1,  0], 
        [ 1,  0,  0], 
        [ 0,  0, -1]
    ],
    [
        [ 1,  0,  0], 
        [ 0, -1,  0], 
        [ 0,  0, -1]
    ],
    [
        [ 0, -1,  0], 
        [-1,  0,  0], 
        [ 0,  0, -1]
    ],
    [
        [ 0,  1,  0], 
        [ 0,  0,  1], 
        [ 1,  0,  0]
    ],
    [
        [ 0,  0,  1], 
        [ 0, -1,  0], 
        [ 1,  0,  0]
    ],
    [
        [ 0, -1,  0], 
        [ 0,  0, -1], 
        [ 1,  0,  0]
    ],
    [
        [ 0,  0, -1], 
        [ 0,  1,  0], 
        [ 1,  0,  0]
    ],
    [
        [ 0, -1,  0], 
        [ 0,  0,  1], 
        [-1,  0,  0]
    ],
    [
        [ 0,  0,  1], 
        [ 0,  1,  0], 
        [-1,  0,  0]
    ],
    [
        [ 0,  1,  0], 
        [ 0,  0, -1], 
        [-1,  0,  0]
    ],
    [
        [ 0,  0, -1], 
        [ 0, -1,  0], 
        [-1,  0,  0]
    ],
    [
        [ 0,  0,  1], 
        [ 1,  0,  0], 
        [ 0,  1,  0]
    ],
    [
        [ 1,  0,  0], 
        [ 0,  0, -1], 
        [ 0,  1,  0]
    ],
    [
        [ 0,  0, -1], 
        [-1,  0,  0], 
        [ 0,  1,  0]
    ],
    [
        [-1,  0,  0], 
        [ 0,  0,  1], 
        [ 0,  1,  0]
    ],
    [
        [ 0,  0, -1], 
        [ 1,  0,  0], 
        [ 0, -1,  0]
    ],
    [
        [ 1,  0,  0], 
        [ 0,  0,  1], 
        [ 0, -1,  0]
    ],
    [
        [ 0,  0,  1], 
        [-1,  0,  0], 
        [ 0, -1,  0]
    ],
    [
        [-1,  0,  0], 
        [ 0,  0, -1], 
        [ 0, -1,  0]
    ],
];
#[derive(Debug, Clone)]
struct ParsedInput {
    scanners: Vec<Scanner>,
}
#[derive(Debug, Clone, Default)]
struct Scanner {
    orient_idx: Option<usize>,
    position: Option<Vec3>,
    beacons: Vec<Vec3>,
}
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Vec3 {
    x: i32,
    y: i32,
    z: i32,
}
impl Vec3 {
    fn distance(&self, rhs: &Vec3) -> u64 {
        ((self.x - rhs.x).abs() + (self.y - rhs.y).abs() + (self.z - rhs.z).abs()) as u64
    }
}
impl From<[i32; 3]> for Vec3 {
    fn from(a: [i32; 3]) -> Self {
        Self {
            x: a[0],
            y: a[1],
            z: a[2],
        }
    }
}
impl std::ops::Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        Self::Output {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}
impl std::ops::Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}
impl std::ops::Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}
impl std::ops::Mul<[[i32; 3]; 3]> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: [[i32; 3]; 3]) -> Self::Output {
        Self::Output {
            x: self.x * rhs[0][0] + self.y * rhs[0][1] + self.z * rhs[0][2],
            y: self.x * rhs[1][0] + self.y * rhs[1][1] + self.z * rhs[1][2],
            z: self.x * rhs[2][0] + self.y * rhs[2][1] + self.z * rhs[2][2],
        }
    }
}
fn common_beacons(scanner1: &Scanner, scanner2: &Scanner) -> Option<(usize, (Vec3, usize))> {
    let s1_pos = scanner1.position?;
    let s1_idx = scanner1.orient_idx?;
    ORIENTATION
        .into_iter()
        .map(|orientation_probe| {
            scanner1
                .beacons
                .iter()
                .cartesian_product(scanner2.beacons.iter())
                .counts_by(|(&beacon_s1, &beacon_s2)| {
                    let b1 = beacon_s1 * ORIENTATION[s1_idx] + s1_pos;
                    let b2 = beacon_s2 * orientation_probe;
                    b1 - b2
                })
                .into_iter()
                .max_by(|(_s2_pos1, count1), (_s2_pos2, count2)| count1.cmp(count2))
                .expect("not empty")
        })
        .enumerate()
        .max_by(|(_orient1, a), (_orient2, b)| a.1.cmp(&b.1))
}
fn detect_scanners_pos(scanners: &mut Vec<Scanner>) {
    let mut stack = vec![0];
    while let Some(idx) = stack.pop() {
        let last = std::mem::take(&mut scanners[idx]);
        for (i, scanner) in scanners
            .iter_mut()
            .enumerate()
            .filter(|(i, scanner)| scanner.position.is_none() && *i != idx)
        {
            let (orient_idx, (pos, count)) = common_beacons(&last, scanner).unwrap();
            if count >= 12 {
                scanner.position = Some(pos);
                scanner.orient_idx = Some(orient_idx);
                stack.push(i);
            }
        }
        scanners[idx] = last;
    }
}
fn unique_beacons(scanners: &[Scanner]) -> usize {
    scanners
        .iter()
        .flat_map(|scanner| {
            scanner.beacons.iter().map(|&beacon| {
                beacon * ORIENTATION[scanner.orient_idx.expect("orientation detected")]
                    + scanner.position.expect("position detected")
            })
        })
        .unique()
        .count()
}
fn largest_manh_distance(scanners: &[Scanner]) -> u64 {
    let positions = scanners.iter().map(|scanner| scanner.position.unwrap());
    positions
        .clone()
        .cartesian_product(positions)
        .map(|(a, b)| a.distance(&b))
        .max()
        .unwrap()
}
fn parse(input: &str) -> anyhow::Result<ParsedInput> {
    let scanner = tuple((tag("--- scanner "), i32, tag(" ---"), line_ending));
    let beacon = separated_list1(char(','), i32).map(|xyz| Vec3 {
        x: xyz[0],
        y: xyz[1],
        z: xyz[2],
    });
    let beacons = separated_list1(line_ending, beacon);
    let scanner_data = delimited(scanner, beacons, line_ending).map(|beacons| Scanner {
        orient_idx: None,
        position: None,
        beacons,
    });
    let mut parser = map(
        separated_list1(line_ending, scanner_data),
        |mut scanners| {
            scanners[0].position = Some(Default::default());
            scanners[0].orient_idx = Some(Default::default());
            ParsedInput { scanners }
        },
    );
    parser
        .parse(input)
        .finish()
        .map(|(_input, parsed)| parsed)
        .map_err(|e: nom::error::VerboseError<&str>| anyhow::anyhow!("parser error: {:?}", e))
}
fn main() -> anyhow::Result<()> {
    let input = read_to_string("day-19/input.txt")?;
    let ParsedInput { mut scanners } = parse(&input)?;
    detect_scanners_pos(&mut scanners);
    let part1 = unique_beacons(&scanners);
    println!("part1 result is {}", part1);
    let part2 = largest_manh_distance(&scanners);
    println!("part2 result is {}", part2);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const INPUT: &str = r#"--- scanner 0 ---
404,-588,-901
528,-643,409
-838,591,734
390,-675,-793
-537,-823,-458
-485,-357,347
-345,-311,381
-661,-816,-575
-876,649,763
-618,-824,-621
553,345,-567
474,580,667
-447,-329,318
-584,868,-557
544,-627,-890
564,392,-477
455,729,728
-892,524,684
-689,845,-530
423,-701,434
7,-33,-71
630,319,-379
443,580,662
-789,900,-551
459,-707,401

--- scanner 1 ---
686,422,578
605,423,415
515,917,-361
-336,658,858
95,138,22
-476,619,847
-340,-569,-846
567,-361,727
-460,603,-452
669,-402,600
729,430,532
-500,-761,534
-322,571,750
-466,-666,-811
-429,-592,574
-355,545,-477
703,-491,-529
-328,-685,520
413,935,-424
-391,539,-444
586,-435,557
-364,-763,-893
807,-499,-711
755,-354,-619
553,889,-390

--- scanner 2 ---
649,640,665
682,-795,504
-784,533,-524
-644,584,-595
-588,-843,648
-30,6,44
-674,560,763
500,723,-460
609,671,-379
-555,-800,653
-675,-892,-343
697,-426,-610
578,704,681
493,664,-388
-671,-858,530
-667,343,800
571,-461,-707
-138,-166,112
-889,563,-600
646,-828,498
640,759,510
-630,509,768
-681,-892,-333
673,-379,-804
-742,-814,-386
577,-820,562

--- scanner 3 ---
-589,542,597
605,-692,669
-500,565,-823
-660,373,557
-458,-679,-417
-488,449,543
-626,468,-788
338,-750,-386
528,-832,-391
562,-778,733
-938,-730,414
543,643,-506
-524,371,-870
407,773,750
-104,29,83
378,-903,-323
-778,-728,485
426,699,580
-438,-605,-362
-469,-447,-387
509,732,623
647,635,-688
-868,-804,481
614,-800,639
595,780,-596

--- scanner 4 ---
727,592,562
-293,-554,779
441,611,-461
-714,465,-776
-743,427,-804
-660,-479,-426
832,-632,460
927,-485,-438
408,393,-506
466,436,-512
110,16,151
-258,-428,682
-393,719,612
-211,-452,876
808,-476,-593
-575,615,604
-485,667,467
-680,325,-822
-627,-443,-432
872,-547,-609
833,512,582
807,604,487
839,-516,451
891,-625,532
-652,-548,-490
30,-46,-14
"#;

    #[test]
    fn part1() -> anyhow::Result<()> {
        let ParsedInput { mut scanners } = parse(INPUT)?;
        detect_scanners_pos(&mut scanners);
        assert_eq!(scanners[1].position.unwrap(), [68, -1246, -43].into());
        assert_eq!(scanners[2].position.unwrap(), [1105, -1205, 1229].into());
        assert_eq!(scanners[3].position.unwrap(), [-92, -2380, -20].into());
        assert_eq!(scanners[4].position.unwrap(), [-20, -1133, 1061].into());
        assert_eq!(unique_beacons(&scanners), 79);

        Ok(())
    }
    #[test]
    fn part2() -> anyhow::Result<()> {
        let ParsedInput { mut scanners } = parse(INPUT)?;
        detect_scanners_pos(&mut scanners);
        assert_eq!(largest_manh_distance(&scanners), 3621);
        Ok(())
    }
}
