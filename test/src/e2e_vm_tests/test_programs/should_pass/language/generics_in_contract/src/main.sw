contract;

struct Game {
    winner: Option<Identity>,
}

abi TicTacToe {
    #[storage(write)]
    fn new_game();
}

storage {
    game: Game = Game { winner: Option::None },
    game_boards: StorageMap<u64, Option<Identity>> = StorageMap {},
}

impl TicTacToe for Contract {
    #[storage(write)]
    fn new_game() {
        storage.game_boards.insert(1, Option::None::<Identity>());
        storage.game_boards.insert(1, Option::None);
    }
}
