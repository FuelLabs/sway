script;

// helper functions for our example
fn on_even(num: u64) {
    // do something with even numbers
}
fn on_odd(num: u64) {
    // do something with odd numbers
}

fn main(num: u64) -> u64 {
    // Match as an expression
    let is_even = match num % 2 {
        0 => true,
        _ => false,
    };

    // Match as control flow
    let x = 12;
    match x {
        5 => on_odd(x),
        _ => on_even(x),
    };

    // Match an enum
    enum Weather {
        Sunny: (),
        Rainy: (),
        Cloudy: (),
        Snowy: (),
    }
    let current_weather = Weather::Sunny;
    let avg_temp = match current_weather {
        Weather::Sunny => 80,
        Weather::Rainy => 50,
        Weather::Cloudy => 60,
        Weather::Snowy => 20,
    };

    let is_sunny = match current_weather {
        Weather::Sunny => true,
        Weather::Rainy | Weather::Cloudy | Weather::Snowy => false,
    };

    // match expression used for a return
    let outside_temp = Weather::Sunny;
    match outside_temp {
        Weather::Sunny => 80,
        Weather::Rainy => 50,
        Weather::Cloudy => 60,
        Weather::Snowy => 20,
    }
}
