library;

use std::outputs::Output;

#[test]
fn output_output_eq() {
    let output_1 = Output::Coin;
    let output_2 = Output::Coin;
    let output_3 = Output::Contract;
    let output_4 = Output::Contract;
    let output_5 = Output::Change;
    let output_6 = Output::Change;
    let output_7 = Output::Variable;
    let output_8 = Output::Variable;
    let output_9 = Output::ContractCreated;
    let output_10 = Output::ContractCreated;

    assert_eq(output_1, output_2);
    assert_eq(output_3, output_4);
    assert_eq(output_5, output_6);
    assert_eq(output_7, output_8);
    assert_eq(output_9, output_10);
}

#[test]
fn output_output_neq() {
    let output_1 = Output::Coin;
    let output_2 = Output::Contract;
    let output_3 = Output::Change;
    let output_4 = Output::Variable;
    let output_5 = Output::ContractCreated;

    assert_ne(output_1, output_2);
    assert_ne(output_1, output_3);
    assert_ne(output_1, output_4);
    assert_ne(output_1, output_5);

    assert_ne(output_2, output_3);
    assert_ne(output_2, output_4);
    assert_ne(output_2, output_5);

    assert_ne(output_3, output_4);
    assert_ne(output_3, output_5);

    assert_ne(output_4, output_5);
}
