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
    let isEven = match num % 2 {
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
    let currentWeather = Weather::Sunny;
    let avgTemp = match currentWeather {
        Weather::Sunny => 80,
        Weather::Rainy => 50,
        Weather::Cloudy => 60,
        Weather::Snowy => 20,
    };

// match expression used for a return
    let outsideTemp = Weather::Sunny;
    match outsideTemp {
        Weather::Sunny => 80,
        Weather::Rainy => 50,
        Weather::Cloudy => 60,
        Weather::Snowy => 20,
    }
}
