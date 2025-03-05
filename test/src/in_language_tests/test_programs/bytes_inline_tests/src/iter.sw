library;

use std::bytes::Bytes;

#[test]
fn bytes_iter() {
    let mut bytes: Bytes = Bytes::new();

    let number0 = 0;
    let number1 = 1;
    let number2 = 2;
    let number3 = 3;
    let number4 = 4;

    bytes.push(number0);
    bytes.push(number1);
    bytes.push(number2);
    bytes.push(number3);
    bytes.push(number4);

    let mut iter = bytes.iter();

    assert(iter.next() == Some(number0));
    assert(iter.next() == Some(number1));
    assert(iter.next() == Some(number2));
    assert(iter.next() == Some(number3));
    assert(iter.next() == Some(number4));
    assert(iter.next() == None);
    assert(iter.next() == None);
}

#[test]
fn bytes_for_loop() {
    let mut bytes: Bytes = Bytes::new();

    let number0 = 0;
    let number1 = 1;
    let number2 = 2;
    let number3 = 3;
    let number4 = 4;

    bytes.push(number0);
    bytes.push(number1);
    bytes.push(number2);
    bytes.push(number3);
    bytes.push(number4);

    let arr = [number0, number1, number2, number3, number4];

    let mut i = 0;
    for elem in bytes.iter() {
        assert(elem == arr[i]);
        i += 1;
    }
}
