script;

mod heaven;
mod earth;
mod hell;

use heaven::UNKNOWN_DEITY_VALUE;
use earth::MAN;
use hell::THE_DEVIL;

fn god() -> u64 {
    if MAN == 5 && THE_DEVIL == 6 {
        7
    } else {
        UNKNOWN_DEITY_VALUE
    }
}

use heaven::MONKEYS_GONE_HERE;

fn main() -> bool {
    if god() == 7 {
        MONKEYS_GONE_HERE
    } else {
        false
    }
}
