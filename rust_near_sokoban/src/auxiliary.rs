use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Copy, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    pub fn get_point_in_direction(&self, direction: Direction) -> Option<Self> {
        match direction {
            Direction::Backward => {
                if self.x != 0 {
                    Some(Point {
                            x: self.x - 1,
                            ..*self
                        })
                } else { None }
            },
            Direction::Forward => Some(Point {
                x: self.x + 1,
                ..*self
            }),
            Direction::Up => {
                if self.x != 0 {
                    Some(Point {
                        y: self.y - 1,
                        ..*self
                    })
                } else { None }
            }, 
            Direction::Down => Some(Point {
                y: self.y + 1,
                ..*self
            }),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Copy, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum Direction {
    Backward,
    Forward,
    Up,
    Down,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum Outcome {
    Success,
    Failure,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Copy, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Size {
    pub width: usize,
    pub height: usize
}