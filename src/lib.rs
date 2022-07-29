use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{near_bindgen, BorshStorageKey, PanicOnDefault};
use near_sdk::collections::Vector;
use near_sdk::json_types::Base64VecU8;

use crate::board::*;
use crate::auxiliary::*;

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    Boards,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub boards: Vector<Board>,
}

pub type BoardIndex = u64;

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            boards: Vector::new(StorageKey::Boards)
        }
    }

    pub fn create_board(&mut self, field: Base64VecU8, field_size: Option<Size>) -> BoardIndex {
        let size = field_size.unwrap_or(Size { width: 8, height: 8 });

        let board = Board::from(field, size);
        let index = self.boards.len();

        self.boards.push(&board);
        index
    }

    pub fn get_board(&self, index: BoardIndex) -> Option<Board> {
        self.boards.get(index)
    }

    pub fn validate_board(&mut self, index: BoardIndex) {
        let board = self.get_board(index).expect("No board");
        let new_board = board.validate_board();
        self.boards.replace(index, &new_board);
    }

    pub fn step(&mut self, index: BoardIndex, direction: Direction) -> Option<Board> {
        let board = self.get_board(index).expect("No board");
        if board.is_valid == true {
            let new_board = board.make_step(direction);
            self.boards.replace(index, &new_board);

            Some(new_board)
        } else {
            None
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new().is_view(is_view).build()
    }

    #[test]
    fn test_new() {
        let context = get_context(false);
        testing_env!(context);
        let mut _contract = Contract::new();
    }

    #[test]
    fn test_board_create_get() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new();


        let width: usize = 8;
        let height: usize = 8;
        let field_len: usize = (width / 2) * height;

        let mut field = vec![0u8; field_len];
        field[0] = 50;
        let index = contract.create_board(field.clone().into(), None);
        assert_eq!(index, 0);

        testing_env!(get_context(true));
        let board = contract.get_board(0).unwrap();
        assert_eq!(field, board.field.0);    
    }
}

pub mod board;
pub mod auxiliary;