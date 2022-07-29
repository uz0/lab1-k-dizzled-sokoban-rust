use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::require;

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

        require!(field.0.len() == field_len, "Passed field_len and passed vector length don't match");
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

    pub fn set_state_at_cell(&mut self, cord: Point, state: u8) {
        let x = cord.x;
        let y = cord.y;

        require!(state <= 6, "There is no such available state");
        require!(x < self.size.width, "Attempt of setting a value beyond the field");
        require!(y < self.size.height, "Attempt of setting a value beyond the field");

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
        board.is_valid = is_valid;

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

    pub fn clone(&self) -> Self {
        Self {
            field: self.field.clone(),
            ..*self
        }
    }
}


#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;

    fn state_as_symbol(state: u8) -> char {
        match state {
            0 => '*',
            1 => '.',
            2 => 'c',
            3 => 'C',
            4 => 's',
            5 => 'S',
            6 => 'X',
            _ => panic!("Invalid map")
        }
    }

    fn get_board_as_string(board: &Board) -> String {
        let mut result = String::from("");

        for i in 0..board.size.height {
            for j in 0..board.size.width {
                let unwraped_cell = board.get_state_at_cell(Point { x: j, y: i }).unwrap();

                let symbol = state_as_symbol(unwraped_cell);

                result.push(symbol);
            }
            result.push('\n');
        }

        result
    }

    #[allow(dead_code)]
    fn debug_board(board: &Board) {
        print!("{}", get_board_as_string(&board));
    }

    #[test]
    fn test_get_set_state_single() {
        let mut board = Board::new(Size { width: 1, height: 1 });

        board.set_state_at_cell(Point { x: 0, y: 0 }, 4);

        let expected_board = String::from("s\n");
        assert_eq!(expected_board, get_board_as_string(&board));
    }

    #[test]
    fn test_get_set_state_one_crate() {
        let mut board = Board::new(Size { width: 4, height: 2 });

        board.set_state_at_cell(Point { x: 0, y: 0 }, 1);
        board.set_state_at_cell(Point { x: 1, y: 0 }, 4);
        board.set_state_at_cell(Point { x: 2, y: 0 }, 2);
        board.set_state_at_cell(Point { x: 3, y: 0 }, 6);
        board.set_state_at_cell(Point { x: 1, y: 1 }, 1);
        board.set_state_at_cell(Point { x: 3, y: 1 }, 5);

        let expected_board = String::from(".scX\n*.*S\n");
        assert_eq!(expected_board, get_board_as_string(&board));
    }

    #[test]
    fn test_get_set_state_all_available() {
        let mut board = Board::new(Size { width: 7, height: 1 });

        board.set_state_at_cell(Point { x: 0, y: 0 }, 0);
        board.set_state_at_cell(Point { x: 1, y: 0 }, 1);
        board.set_state_at_cell(Point { x: 2, y: 0 }, 2);
        board.set_state_at_cell(Point { x: 3, y: 0 }, 3);
        board.set_state_at_cell(Point { x: 4, y: 0 }, 4);
        board.set_state_at_cell(Point { x: 5, y: 0 }, 5);
        board.set_state_at_cell(Point { x: 6, y: 0 }, 6);

        let expected_board = String::from("*.cCsSX\n");
        assert_eq!(expected_board, get_board_as_string(&board));
    }

    #[test]
    #[should_panic]
    fn test_set_unavailable_state() {
        let mut board = Board::new(Size { width: 1, height: 1 });

        board.set_state_at_cell(Point { x: 0, y: 0 }, 7);
    }

    #[test]
    fn test_get_state_at_unavailable_position() {
        let board = Board::new(Size { width: 2, height: 2 });

        assert_eq!(Option::None, board.get_state_at_cell(Point { x: 3, y: 0}));
        assert_eq!(Option::None, board.get_state_at_cell(Point { x: 0, y: 3}));
        assert_eq!(Option::None, board.get_state_at_cell(Point { x: 3, y: 3}));
    }

    #[test]
    fn test_try_validate_invalid_board_two_sokobans() {
        let mut board = Board::new(Size { width: 9, height: 1 });

        board.set_state_at_cell(Point { x: 0, y: 0 }, 4);
        board.set_state_at_cell(Point { x: 1, y: 0 }, 5);

        board = board.validate_board();
        assert_eq!(false, board.is_valid);
    }

    #[test]
    fn test_try_validate_invalid_board_boxes_dests_not_eq() {
        let mut board = Board::new(Size { width: 9, height: 1 });

        board.set_state_at_cell(Point { x: 0, y: 0 }, 4);
        board.set_state_at_cell(Point { x: 1, y: 0 }, 2);

        board = board.validate_board();
        assert_eq!(false, board.is_valid);
    }

    #[test]
    fn test_try_validate_valid_board() {
        let mut board = Board::new(Size { width: 9, height: 1 });

        board.set_state_at_cell(Point { x: 0, y: 0 }, 4);
        board.set_state_at_cell(Point { x: 1, y: 0 }, 2);
        board.set_state_at_cell(Point { x: 2, y: 0 }, 6);

        board = board.validate_board();
        assert_eq!(true, board.is_valid);
    }

    #[test]
    fn test_make_one_step_move_box_on_destination() {
        let mut board = Board::new(Size { width: 4, height: 2 });

        board.set_state_at_cell(Point { x: 0, y: 0 }, 1);
        board.set_state_at_cell(Point { x: 1, y: 0 }, 4);
        board.set_state_at_cell(Point { x: 2, y: 0 }, 2);
        board.set_state_at_cell(Point { x: 3, y: 0 }, 6);
        board.set_state_at_cell(Point { x: 1, y: 1 }, 1);

        board = board.validate_board();
        board = board.make_step(Direction::Forward);

        let expected_board = String::from("..sC\n*.**\n");
        assert_eq!(expected_board, get_board_as_string(&board));
    }

    #[test]
    fn test_make_one_step_move_box_from_destionation() {
        let mut board = Board::new(Size { width: 4, height: 2 });

        board.set_state_at_cell(Point { x: 0, y: 0 }, 1);
        board.set_state_at_cell(Point { x: 1, y: 0 }, 4);
        board.set_state_at_cell(Point { x: 2, y: 0 }, 3);
        board.set_state_at_cell(Point { x: 3, y: 0 }, 1);
        board.set_state_at_cell(Point { x: 1, y: 1 }, 1);

        board = board.validate_board();
        board = board.make_step(Direction::Forward);

        let expected_board = String::from("..Sc\n*.**\n");
        assert_eq!(expected_board, get_board_as_string(&board));
    }

    #[test]
    fn test_make_one_step_move_sokoban() {
        let mut board = Board::new(Size { width: 2, height: 1 });

        board.set_state_at_cell(Point { x: 0, y: 0 }, 4);
        board.set_state_at_cell(Point { x: 1, y: 0 }, 1);

        board = board.validate_board();
        board = board.make_step(Direction::Forward);

        let expected_board = String::from(".s\n");
        assert_eq!(expected_board, get_board_as_string(&board));
    }

    #[test]
    fn test_make_one_step_move_sokoban_on_destination() {
        let mut board = Board::new(Size { width: 3, height: 1 });

        board.set_state_at_cell(Point { x: 0, y: 0 }, 4);
        board.set_state_at_cell(Point { x: 1, y: 0 }, 6);
        board.set_state_at_cell(Point { x: 2, y: 0 }, 2);

        board = board.validate_board();
        board = board.make_step(Direction::Forward);

        let expected_board = String::from(".Sc\n");
        assert_eq!(expected_board, get_board_as_string(&board));
    }

    #[test]
    #[should_panic]
    fn test_make_one_step_move_sokoban_out_of_field_left() {
        let mut board = Board::new(Size { width: 1, height: 1 });

        board.set_state_at_cell(Point { x: 0, y: 0 }, 4);
       
        board = board.validate_board();
        board.make_step(Direction::Backward);
    }

    #[test]
    fn test_make_one_step_move_sokoban_out_of_field_right() {
        let mut board = Board::new(Size { width: 1, height: 1 });

        board.set_state_at_cell(Point { x: 0, y: 0 }, 4);
       
        board = board.validate_board();
        board = board.make_step(Direction::Forward);

        let expected_board = String::from("s\n");
        assert_eq!(expected_board, get_board_as_string(&board));
    }

    #[test]
    fn test_run_simple_game() {
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

        let actions = [
            Direction::Forward, 
            Direction::Forward,
            Direction::Backward,
            Direction::Down
        ];

        let game_states = [
            String::from(".scX*\n.c..*\n.X.C*\n*****\n"), 
            String::from("..sC*\n.c..*\n.X.C*\n*****\n"), 
            String::from(".s.C*\n.c..*\n.X.C*\n*****\n"),
            String::from("...C*\n.s..*\n.C.C*\n*****\n")
        ];

        for (index, action) in actions.iter().enumerate() {
            board = board.make_step(*action);
            assert_eq!(game_states[index], get_board_as_string(&board));
        }   
    }
}
