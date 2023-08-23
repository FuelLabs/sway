contract;

use std::hash::*;

struct Game {
    winner: Option<Identity>,
}

abi TicTacToe {
    #[storage(write)]
    fn new_game();
}

storage {
    game: Game = Game { winner: None },
    game_boards: StorageMap<u64, Option<Identity>> = StorageMap::<u64, Option<Identity>> {},
}

impl TicTacToe for Contract {
    #[storage(write)]
    fn new_game() {
        storage.game_boards.insert(1, None::<Identity>);
        storage.game_boards.insert(1, None);
    }
}
