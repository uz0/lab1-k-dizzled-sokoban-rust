use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, BorshStorageKey, PanicOnDefault};
use near_sdk::collections::Vector;
use near_sdk::json_types::Base64VecU8;

use crate::auxiliary::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Board {
    pub field: Base64VecU8,
    pub is_valid: bool,
    pub sokoban_position: Option<Point>, 
    pub size: Size, 
    pub field_len: usize,
}

impl Board {
    pub fn new(size: Size) -> Self {
        let mut field_len = size.width * size.height;
        if field_len % 2 == 1 {
            field_len += 1;
        }

        field_len /= 2;

        Self {
            field: Base64VecU8::from(vec![0u8; field_len]),
            is_valid: false,
            sokoban_position: Option::None,
            field_len,
            size,
        }
    }

    pub fn from(field: Base64VecU8, size: Size) -> Self {
        let mut field_len = size.width * size.height;
        if field_len % 2 == 1 {
            field_len += 1;
        }

        field_len /= 2;

        assert_eq!(field.0.len(), field_len);
        let board = Self {
            field, 
            is_valid: false,
            sokoban_position: Option::None,
            field_len,
            size,
        };

        board.validate_board()
    }

    pub fn get_state_at_cell(&self, cord: Point) -> Option<u8> {
        let x = cord.x;
        let y = cord.y;

        if x >= self.size.width || y >= self.size.height {
            return Option::None 
        }

        let cell_index = y * self.size.width + x;
        let in_vector_index = cell_index / 2;
        let in_u8_index = cell_index % 2;

        if in_u8_index == 1 {
            Some(self.field.0[in_vector_index] & 0x0F)
        } else {
            Some(self.field.0[in_vector_index] >> 4)
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            field: self.field.clone(),
            ..*self
        }
    }

    pub fn set_state_at_cell(&mut self, cord: Point, state: u8) {
        let x = cord.x;
        let y = cord.y;

        assert!(state <= 6);
        assert!(x < self.size.width);
        assert!(y < self.size.height);

        let cell_index = y * self.size.width + x;
        let in_vector_index = cell_index / 2;
        let in_u8_index = cell_index % 2;

        let mut value: u8 = self.field.0[in_vector_index];

        if in_u8_index == 1 {
            value &= 0b11110000;
            value |= state;
        } else {
            value = (state << 4) | (value & 15);
        }

        self.field.0[in_vector_index] = value;
    }

    pub fn validate_board(&self) -> Self {
        let mut board : Board = self.clone();

        let mut sokoban_counter = 0;
        let mut box_counter = 0; 
        let mut dest_counter = 0;

        let mut sokoban_position: Point = Point { x: 0, y: 0 };

        for x in 0..self.size.width {
            for y in 0..self.size.height {
                match board.get_state_at_cell(Point { x, y }).unwrap() {
                    2 => box_counter += 1,
                    4 => { 
                        sokoban_counter += 1;
                        sokoban_position.x = x;
                        sokoban_position.y = y;
                    }
                    5 => {
                        sokoban_counter += 1;
                        dest_counter += 1;
                        sokoban_position.x = x;
                        sokoban_position.y = y;
                    },
                    6 => dest_counter +=1,
                    _ => ()
                };
            }
        }

        let is_valid = sokoban_counter == 1 && box_counter == dest_counter;
        board.is_valid = true;
        if is_valid {
            board.sokoban_position = Option::Some(sokoban_position);
        }

        board
    }

    pub fn make_step(&self, direction: Direction) -> Self {
        let mut board = Board {
            field: self.field.clone(),
            ..*self
        };

        let cur_cell = board.sokoban_position.expect("Invalid board");

        let next_cell = match direction {
            Direction::Backward => Point {
                x: cur_cell.x - 1,
                ..cur_cell
            },
            Direction::Forward => Point {
                x: cur_cell.x + 1,
                ..cur_cell
            },
            Direction::Up => Point {
                y: cur_cell.y - 1,
                ..cur_cell
            }, 
            Direction::Down => Point {
                y: cur_cell.y + 1,
                ..cur_cell
            },
        };
        let after_next_cell = match direction {
            Direction::Backward => Point {
                x: next_cell.x - 1,
                ..next_cell
            }, 
            Direction::Forward => Point {
                x: next_cell.x + 1,
                ..next_cell
            },
            Direction::Up => Point {
                y: next_cell.y - 1,
                ..next_cell
            }, 
            Direction::Down => Point {
                y: next_cell.y + 1,
                ..next_cell
            },
        };
        if let Some(state_at_next_cell) = board.get_state_at_cell(next_cell) {
            match state_at_next_cell {
                1 => {
                    board.set_state_at_cell(next_cell, 4);
                    board.sokoban_position = Some(next_cell);
                    board.set_state_at_cell(cur_cell, 1);
                },
                2 => {
                    let state = board.get_state_at_cell(after_next_cell);

                    if let Some(state) = state {
                        if state == 1 {
                            board.set_state_at_cell(next_cell, 4);
                            board.sokoban_position = Some(next_cell);
                            board.set_state_at_cell(cur_cell, 1);
                            board.set_state_at_cell(after_next_cell, 2)
                        } else if state == 6 {
                            board.set_state_at_cell(next_cell, 4);
                            board.sokoban_position = Some(next_cell);
                            board.set_state_at_cell(cur_cell, 1);
                            board.set_state_at_cell(after_next_cell, 3)
                        }
                    } 
                },
                3 => {
                    let state = board.get_state_at_cell(after_next_cell);

                    if let Some(state) = state {
                        if state == 1 {
                            board.set_state_at_cell(next_cell, 5);
                            board.sokoban_position = Some(next_cell);
                            board.set_state_at_cell(cur_cell, 1);
                            board.set_state_at_cell(after_next_cell, 2)
                        } else if state == 6 {
                            board.set_state_at_cell(next_cell, 5);
                            board.sokoban_position = Some(next_cell);
                            board.set_state_at_cell(cur_cell, 1);
                            board.set_state_at_cell(after_next_cell, 3)
                        }
                    }
                }
                6 => {
                    board.set_state_at_cell(next_cell, 5);
                    board.sokoban_position = Some(next_cell);
                    board.set_state_at_cell(cur_cell, 1);
                },
                _ => ()
            };
        } 

        board 
    }
}


#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;

    fn debug_board(board: &Board) {
        for i in 0..board.size.height {
            for j in 0..board.size.width {
                let unwraped_cell = board.get_state_at_cell(Point { x: j, y: i }).unwrap();

                let symbol = match unwraped_cell {
                    0 => '*',
                    1 => '.',
                    2 => 'c',
                    3 => 'C',
                    4 => 's',
                    5 => 'S',
                    6 => 'X',
                    _ => panic!("Invalid map")
                };

                print!("{symbol}");
            }
            println!();
        }
    }

    #[test]
    fn test_one_step() {
        let mut board = Board::new(Size { width: 5, height: 4 });

        board.set_state_at_cell(Point { x: 0, y: 0 }, 4);
        board.set_state_at_cell(Point { x: 1, y: 0 }, 1);
        board.set_state_at_cell(Point { x: 2, y: 0 }, 2);
        board.set_state_at_cell(Point { x: 3, y: 0 }, 6);
        board.set_state_at_cell(Point { x: 0, y: 1 }, 1);
        board.set_state_at_cell(Point { x: 1, y: 1 }, 2);
        board.set_state_at_cell(Point { x: 2, y: 1 }, 1);
        board.set_state_at_cell(Point { x: 3, y: 1 }, 1);
        board.set_state_at_cell(Point { x: 0, y: 2 }, 1);
        board.set_state_at_cell(Point { x: 1, y: 2 }, 6);
        board.set_state_at_cell(Point { x: 2, y: 2 }, 1);
        board.set_state_at_cell(Point { x: 3, y: 2 }, 3);

        let mut board = board.validate_board();

        debug_board(&board); 
        println!();

        let actions = [
            Direction::Forward, 
            Direction::Forward,
            Direction::Backward,
            Direction::Down
        ];

        for action in actions {
            board = board.make_step(action);
            debug_board(&board); 

            println!();
        }   
    }
}
