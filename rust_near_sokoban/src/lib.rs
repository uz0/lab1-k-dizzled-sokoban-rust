use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, require, AccountId, near_bindgen, BorshStorageKey, PanicOnDefault};
use near_sdk::collections::{Vector, LookupMap};
use near_sdk::json_types::Base64VecU8;

use crate::board::*;
use crate::auxiliary::*;
use crate::game::*;

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    Boards,
    Games, 
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub boards: Vector<Board>,
    pub games: Vector<SingleplayerGame>,
}

pub type BoardIndex = u64;
pub type GameIndex = u64;

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            boards: Vector::new(StorageKey::Boards),
            games: Vector::new(StorageKey::Games),
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

    pub fn create_single_game(
        &mut self, 
        index: BoardIndex, 
        player: AccountId,
    ) -> GameIndex {
        let board = self.get_board(index).expect("No board");
        require!(board.is_valid, "Invalid board to play!");

        let board = board.clone();

        let game = SingleplayerGame::from(board, player);
        let index = self.games.len();

        self.games.push(&game);
        index
    }

    pub fn get_single_game(&self, index: GameIndex) -> Option<SingleplayerGame> {
        self.games.get(index)
    }

    pub fn start_single_game(&mut self, index: GameIndex) {
        let mut game = self.get_single_game(index).expect("Game doesn't exist");
        game.game_status = GameStatus::Running;
        self.games.replace(index, &game);
    }

    pub fn step(&mut self, index: GameIndex, direction: Direction) -> SingleplayerGame {
        let mut game = self.games
            .get(index)
            .expect("Game doesn't exist");

        let old_board = game.board.clone();
        env::log_str("Old board");
        old_board.debug_logs();

        game.make_step(direction);

        env::log_str("New board");
        game.board.debug_logs();

        self.games.replace(index, &game);
        return self.games.get(index).unwrap();
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, VMContext};

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new().is_view(is_view).build()
    }

    fn get_context_account(account: AccountId) -> near_sdk::VMContext {
        VMContextBuilder::new()
            .predecessor_account_id(account)
            .build()
    }

    impl PartialEq for Board {
        fn eq(&self, other: &Self) -> bool {
            self.field == other.field && 
            self.is_valid == other.is_valid && 
            self.sokoban_position == other.sokoban_position && 
            self.size == other.size && 
            self.field_len == other.field_len
        }
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

    #[test]
    fn test_single_game_create_get() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new();

        let mut board = Board::new(Size { width: 2, height: 1 });
        board.set_state_at_cell(Point { x: 0, y: 0 }, 4);
        board.set_state_at_cell(Point { x: 1, y: 0 }, 1);

        let index = contract.create_board(
            board.field.clone(), 
            Some(Size { width: 2, height: 1 })
        );

        let game_index = contract.create_single_game(index, accounts(0));
        assert_eq!(game_index, 0);

        let game = contract.get_single_game(game_index);

        assert!(contract.get_single_game(game_index + 1).is_none());
        assert!(game.is_some());
        assert_eq!(game.as_ref().unwrap().player, accounts(0));
        assert_eq!(game.unwrap().board.field, board.field);
    }

    #[test]
    fn test_make_move() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new();

        let mut board = Board::new(Size { width: 4, height: 2 });
        board.set_state_at_cell(Point { x: 0, y: 0 }, 1);
        board.set_state_at_cell(Point { x: 1, y: 0 }, 4);
        board.set_state_at_cell(Point { x: 2, y: 0 }, 2);
        board.set_state_at_cell(Point { x: 3, y: 0 }, 6);
        board.set_state_at_cell(Point { x: 1, y: 1 }, 1);

        let index = contract.create_board(
            board.field.clone(), 
            Some(Size { width: 4, height: 2 })
        );

        let game_index = contract.create_single_game(index, accounts(0));
        let game = contract.get_single_game(game_index);
        assert_eq!(game.unwrap().game_status, GameStatus::Unactive);

        contract.start_single_game(game_index);
        let game = contract.get_single_game(game_index);
        assert_eq!(game.unwrap().game_status, GameStatus::Running);

        testing_env!(get_context_account(accounts(0)));
        contract.step(game_index, Direction::Forward);
        let game = contract.get_single_game(game_index);
        assert_eq!(game.unwrap().game_status, GameStatus::Finished);
    }
}

pub mod board;
pub mod auxiliary;
pub mod game;