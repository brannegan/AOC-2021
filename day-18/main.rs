use std::fs::read_to_string;

use indextree::{Arena, NodeId};
use itertools::Itertools;
use nom::character::complete::{char, u32};

use nom::combinator::map;
use nom::multi::separated_list1;
use nom::sequence::delimited;
use nom::{Finish, IResult, Parser};

#[derive(Debug, Clone)]
struct FishNum {
    arena: Arena<Option<u32>>,
    root: Option<NodeId>,
}

impl FishNum {
    fn new(input: &str) -> Self {
        let arena = Arena::new();
        let mut this = Self { arena, root: None };
        let _ = this.parse(None, input);
        this
    }
    fn parse_num<'i>(&mut self, parent: NodeId, input: &'i str) -> IResult<&'i str, ()> {
        let num: Result<_, nom::error::Error<&str>> = map(u32, |n| {
            let node = self.new_node(Some(n));
            parent.append(node, self);
        })
        .parse(input)
        .finish();
        num.or_else(|_| self.parse(Some(parent), input))
    }
    fn parse<'i>(&mut self, parent: Option<NodeId>, input: &'i str) -> IResult<&'i str, ()> {
        let new_node = self.new_node(None);
        if let Some(parent) = parent {
            parent.append(new_node, self);
        } else {
            self.root = Some(new_node);
        }
        let num_or_pair = |i| self.parse_num(new_node, i);

        let mut parser = delimited(
            char('['),
            separated_list1(char(','), num_or_pair),
            char(']'),
        )
        .map(drop);
        parser.parse(input)
    }
    fn split(&mut self) -> bool {
        let split_node = self
            .root
            .expect("some root")
            .descendants(self)
            .find(|id| self[*id].get().gt(&Some(9)));

        match split_node {
            Some(split_node) => {
                let num = self[split_node]
                    .get_mut()
                    .take()
                    .expect("leaf nodes are values");
                let left = self.new_node(Some(num / 2));
                let right = self.new_node(Some(num / 2 + (num % 2)));
                split_node.append(left, self);
                split_node.append(right, self);
                true
            }
            None => false,
        }
    }
    fn explode(&mut self) -> bool {
        let explode_node = self.explode_node(self.root.expect("some root"), 4);
        if explode_node.is_none() {
            return false;
        }
        let explode_node = unsafe { explode_node.unwrap_unchecked() };

        let nums: Vec<u32> = explode_node
            .children(self)
            .map(|child| (*self[child].get()).expect("leaf node"))
            .collect();
        if let Some(left) = explode_node.preceding_siblings(self).nth(1) {
            let left_leaf = left
                .descendants(self)
                .find(|&leaf| self[leaf].get().is_some())
                .expect("leaf node");
            *self[left_leaf]
                .get_mut()
                .as_mut()
                .expect("leaf nodes are values") += nums[0];
            let mut right_node = explode_node;
            let right_branch = explode_node
                .ancestors(self)
                .skip(1)
                .find(|parent| {
                    let found = right_node != parent.children(self).nth(1).expect("right child");
                    right_node = *parent;
                    found
                })
                .map(|node| node.children(self).nth(1).expect("right child"));
            if let Some(right_branch) = right_branch {
                let right_leaf = right_branch
                    .descendants(self)
                    .find(|&leaf| self[leaf].get().is_some())
                    .expect("first preorder leaf node");
                *self[right_leaf]
                    .get_mut()
                    .as_mut()
                    .expect("leaf nodes are values") += nums[1];
            }
        }
        if let Some(right) = explode_node.following_siblings(self).nth(1) {
            let right_leaf = right
                .descendants(self)
                .find(|&leaf| self[leaf].get().is_some())
                .expect("leaf node");
            *self[right_leaf]
                .get_mut()
                .as_mut()
                .expect("leaf nodes are values") += nums[1];
            let mut left_node = explode_node;
            let left_branch = explode_node
                .ancestors(self)
                .skip(1)
                .find(|parent| {
                    let found = left_node != parent.children(self).next().expect("left child");
                    left_node = *parent;
                    found
                })
                .map(|node| node.children(self).next().expect("left child"));
            if let Some(left_branch) = left_branch {
                let left_leaf = left_branch
                    .descendants(self)
                    .last()
                    .expect("last preorder leaf node");
                *self[left_leaf]
                    .get_mut()
                    .as_mut()
                    .expect("leaf nodes are values") += nums[0];
            }
        }
        explode_node.insert_before(self.new_node(Some(0)), self);
        explode_node.remove_subtree(self);
        true
    }
    fn reduce(&mut self) {
        while self.explode() || self.split() {}
    }
    fn magnitude(&self) -> u32 {
        self.magnitude_node(self.root.expect("some root"))
    }
    fn magnitude_node(&self, node: NodeId) -> u32 {
        if let Some(num) = self[node].get() {
            return *num;
        }
        let left = node.children(self).next().expect("left child");
        let right = node.children(self).nth(1).expect("right child");
        3 * self.magnitude_node(left) + 2 * self.magnitude_node(right)
    }
    fn explode_node(&self, node: NodeId, mut height: i32) -> Option<NodeId> {
        if height == 0 && self[node].get().is_none() {
            return Some(node);
        }
        height -= 1;
        node.children(self)
            .flat_map(|child| self.explode_node(child, height))
            .take(1)
            .next()
    }
    fn format_node(&self, node: NodeId, s: &mut String) {
        s.push('[');
        if let Some(num) = self[node].get() {
            s.push_str(&u32::to_string(num));
            s.push(']');
            return;
        }
        let mut first = true;
        for child in node.children(self) {
            if first {
                first = false;
            } else {
                s.push(',');
            }
            if let Some(num) = self[child].get() {
                s.push_str(&u32::to_string(num));
            } else {
                self.format_node(child, s);
            }
        }
        s.push(']');
    }
}
impl std::ops::Deref for FishNum {
    type Target = Arena<Option<u32>>;

    fn deref(&self) -> &Self::Target {
        &self.arena
    }
}
impl std::ops::DerefMut for FishNum {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.arena
    }
}
impl std::fmt::Display for FishNum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        self.format_node(self.root.expect("some root"), &mut s);
        write!(f, "{s}")
    }
}
impl std::ops::AddAssign for FishNum {
    fn add_assign(&mut self, rhs: Self) {
        let new_root = self.new_node(None);
        new_root.append(self.root.expect("some root"), self);
        self.root = Some(new_root);
        let _ = self.parse(self.root, &rhs.to_string());
    }
}
fn main() {
    let input = read_to_string("day-18/input.txt").expect("file exists");
    let part1 = input
        .trim()
        .split('\n')
        .map(FishNum::new)
        .reduce(|mut acc, num| {
            acc += num;
            acc.reduce();
            acc
        })
        .expect("not empty")
        .magnitude();
    println!("part1 result is {}", part1);
    let part2 = input
        .trim()
        .split('\n')
        .map(FishNum::new)
        .permutations(2)
        .map(|pair| {
            pair.into_iter()
                .reduce(|mut acc, num| {
                    acc += num;
                    acc.reduce();
                    acc
                })
                .expect("not empty")
                .magnitude()
        })
        .max()
        .expect("not empty");
    println!("part2 result is {}", part2);
}
#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn add() {
        let mut num = FishNum::new("[[1,2],3]");
        let num2 = FishNum::new("[4,5]");
        num += num2;
        assert_eq!("[[[1,2],3],[4,5]]", num.to_string());
    }
    #[test]
    fn split() {
        let mut num = FishNum::new("[11,[1,1]]");
        num.split();
        assert_eq!(num.to_string(), FishNum::new("[[5,6],[1,1]]").to_string());
        let mut num = FishNum::new("[[1,1],11]");
        num.split();
        assert_eq!(num.to_string(), FishNum::new("[[1,1],[5,6]]").to_string());
        let mut num = FishNum::new("[[11,11],1]");
        num.split();
        assert_eq!(num.to_string(), FishNum::new("[[[5,6],11],1]").to_string());
        num.split();
        assert_eq!(
            num.to_string(),
            FishNum::new("[[[5,6],[5,6]],1]").to_string()
        );
    }
    #[test]
    fn explode() {
        let mut num = FishNum::new("[[[[[9,8],1],2],3],4]");
        num.explode();
        assert_eq!(num.to_string(), "[[[[0,9],2],3],4]");

        let mut num = FishNum::new("[7,[6,[5,[4,[3,2]]]]]");
        num.explode();
        assert_eq!(num.to_string(), "[7,[6,[5,[7,0]]]]");

        let mut num = FishNum::new("[[6,[5,[4,[3,2]]]],1]");
        num.explode();
        assert_eq!(num.to_string(), "[[6,[5,[7,0]]],3]");

        let mut num = FishNum::new("[[3,[2,[1,[7,3]]]],[6,[5,[4,[3,2]]]]]");
        num.explode();
        assert_eq!(num.to_string(), "[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]");

        let mut num = FishNum::new("[[3,[2,[8,0]]],[9,[5,[4,[3,2]]]]]");
        num.explode();
        assert_eq!(num.to_string(), "[[3,[2,[8,0]]],[9,[5,[7,0]]]]");

        let mut num = FishNum::new("[[[[0,7],4],[7,[[8,4],9]]],[1,1]]");
        num.explode();
        assert_eq!(num.to_string(), "[[[[0,7],4],[15,[0,13]]],[1,1]]");

        let mut num = FishNum::new("[[[[0,7],4],[[7,8],[0,[6,7]]]],[1,1]]");
        num.explode();
        assert_eq!(num.to_string(), "[[[[0,7],4],[[7,8],[6,0]]],[8,1]]");
    }
    #[test]
    fn reduce() {
        let mut num = FishNum::new("[[[[4,3],4],4],[7,[[8,4],9]]]");
        num += FishNum::new("[1,1]");
        num.reduce();
        assert_eq!(num.to_string(), "[[[[0,7],4],[[7,8],[6,0]]],[8,1]]");

        let mut num = FishNum::new("[[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]]");
        num += FishNum::new("[7,[[[3,7],[4,3]],[[6,3],[8,8]]]]");
        num.reduce();
        assert_eq!(
            num.to_string(),
            "[[[[4,0],[5,4]],[[7,7],[6,0]]],[[8,[7,7]],[[7,9],[5,0]]]]"
        );
    }
    #[test]
    fn multi() {
        let input = r#"
[[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]]
[7,[[[3,7],[4,3]],[[6,3],[8,8]]]]
[[2,[[0,8],[3,4]]],[[[6,7],1],[7,[1,6]]]]
[[[[2,4],7],[6,[0,5]]],[[[6,8],[2,8]],[[2,1],[4,5]]]]
[7,[5,[[3,8],[1,4]]]]
[[2,[2,2]],[8,[8,1]]]
[2,9]
[1,[[[9,3],9],[[9,0],[0,7]]]]
[[[5,[7,4]],7],1]
[[[[4,2],2],6],[8,7]]
        "#
        .trim();
        let result = input
            .split('\n')
            .map(FishNum::new)
            .reduce(|mut acc, num| {
                acc += num;
                acc.reduce();
                acc
            })
            .unwrap();

        assert_eq!(
            result.to_string(),
            "[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]"
        );
    }
    #[test]
    fn magnitude() {
        assert_eq!(FishNum::new("[[1,2],[[3,4],5]]").magnitude(), 143);
        assert_eq!(
            FishNum::new("[[[[0,7],4],[[7,8],[6,0]]],[8,1]]").magnitude(),
            1384
        );
        assert_eq!(
            FishNum::new("[[[[1,1],[2,2]],[3,3]],[4,4]]").magnitude(),
            445
        );
        assert_eq!(
            FishNum::new("[[[[3,0],[5,3]],[4,4]],[5,5]]").magnitude(),
            791
        );
        assert_eq!(
            FishNum::new("[[[[5,0],[7,4]],[5,5]],[6,6]]").magnitude(),
            1137
        );
        assert_eq!(
            FishNum::new("[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]").magnitude(),
            3488
        );
    }
    #[test]
    fn final_example() {
        let input = r#"
[[[0,[5,8]],[[1,7],[9,6]]],[[4,[1,2]],[[1,4],2]]]
[[[5,[2,8]],4],[5,[[9,9],0]]]
[6,[[[6,2],[5,6]],[[7,6],[4,7]]]]
[[[6,[0,7]],[0,9]],[4,[9,[9,0]]]]
[[[7,[6,4]],[3,[1,3]]],[[[5,5],1],9]]
[[6,[[7,3],[3,2]]],[[[3,8],[5,7]],4]]
[[[[5,4],[7,7]],8],[[8,3],8]]
[[9,3],[[9,9],[6,[4,9]]]]
[[2,[[7,7],7]],[[5,8],[[9,3],[0,2]]]]
[[[[5,2],5],[8,[3,7]]],[[5,[7,5]],[4,4]]]
        "#
        .trim();
        let result = input
            .split('\n')
            .map(FishNum::new)
            .reduce(|mut acc, num| {
                acc += num;
                acc.reduce();
                acc
            })
            .unwrap();

        assert_eq!(result.magnitude(), 4140);
    }
    #[test]
    fn part2() {
        let input = r#"
[[[0,[5,8]],[[1,7],[9,6]]],[[4,[1,2]],[[1,4],2]]]
[[[5,[2,8]],4],[5,[[9,9],0]]]
[6,[[[6,2],[5,6]],[[7,6],[4,7]]]]
[[[6,[0,7]],[0,9]],[4,[9,[9,0]]]]
[[[7,[6,4]],[3,[1,3]]],[[[5,5],1],9]]
[[6,[[7,3],[3,2]]],[[[3,8],[5,7]],4]]
[[[[5,4],[7,7]],8],[[8,3],8]]
[[9,3],[[9,9],[6,[4,9]]]]
[[2,[[7,7],7]],[[5,8],[[9,3],[0,2]]]]
[[[[5,2],5],[8,[3,7]]],[[5,[7,5]],[4,4]]]
        "#
        .trim();

        let result = input
            .split('\n')
            .map(FishNum::new)
            .permutations(2)
            .map(|pair| {
                pair.into_iter()
                    .reduce(|mut acc, num| {
                        acc += num;
                        acc.reduce();
                        acc
                    })
                    .unwrap()
                    .magnitude()
            })
            .max()
            .unwrap();

        assert_eq!(result, 3993);
    }
}
