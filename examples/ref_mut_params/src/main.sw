script;

use std::assert::assert;

enum Color {
    Red: (),
    Blue: (),
}

// ANCHOR: increment
fn increment(ref mut num: u32) {
    let prev = num;
    num = prev + 1u32;
}
// ANCHOR_END: increment

// ANCHOR: tuple_and_enum
fn swap_tuple(ref mut pair: (u64, u64)) {
    let temp = pair.0;
    pair.0 = pair.1;
    pair.1 = temp;
}

fn update_color(ref mut color: Color, new_color: Color) {
    color = new_color;
}
// ANCHOR_END: tuple_and_enum

// ANCHOR: move_right
struct Coordinates {
    x: u64,
    y: u64,
}

impl Coordinates {
    fn move_right(ref mut self, distance: u64) {
        self.x += distance;
    }
}
// ANCHOR_END: move_right

fn main() {
    // ANCHOR: call_increment
    let mut num: u32 = 0;
    increment(num);
    assert(num == 1u32); // The function `increment()` modifies `num`
    // ANCHOR_END: call_increment

    // ANCHOR: call_tuple_and_enum
    let mut tuple = (42, 24);
    swap_tuple(tuple);
    assert(tuple.0 == 24); // The function `swap_tuple()` modifies `tuple.0`
    assert(tuple.1 == 42); // The function `swap_tuple()` modifies `tuple.1`

    let mut color = Color::Red;
    update_color(color, Color::Blue);
    assert(match color {
        Color::Blue => {
            true
        }
        _ => {
            false
        }
    }
    ); // The function `update_color()` modifies the color to Blue
    // ANCHOR_END: call_tuple_and_enum

    // ANCHOR: call_move_right
    let mut point = Coordinates {
        x: 1,
        y: 1,
    };
    point.move_right(5);
    assert(point.x == 6);
    assert(point.y == 1);
    // ANCHOR_END: call_move_right
}
