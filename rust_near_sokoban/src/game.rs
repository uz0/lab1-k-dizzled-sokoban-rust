use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, require, AccountId};

use crate::board::*;
use crate::auxiliary::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Player {
	pub account_id: AccountId, // near account id eg 'player1.near'
	pub roketo_stream: String, // near sdk CryptoHash
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum GameStatus {
	Unactive,
	Running, 
	Finished,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SingleplayerGame {
	pub board: Board,
	pub player: AccountId,
	pub game_status: GameStatus, 
}

impl SingleplayerGame {
	pub fn from(board: Board, player: AccountId) -> Self {
        Self {
            board, 
            player, 
            game_status: GameStatus::Unactive,
        }
    }

    pub fn make_step(&mut self, direction: Direction) {
        // Require game status is correсt
        require!(self.game_status != GameStatus::Finished, "Game is already finished!");
        require!(self.game_status != GameStatus::Unactive, "Game has not been started yet!");
        // Require player valid
        require!(
            env::predecessor_account_id() == self.player,
            "Incorrect predecessor account"
        );

        let new_board = self.board.make_step(direction);

        self.board = new_board.clone();
        if new_board.check_if_finished() {
        	self.game_status = GameStatus::Finished;
        }
    }
}
