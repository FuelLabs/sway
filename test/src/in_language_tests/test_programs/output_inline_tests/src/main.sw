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

    assert(output_1 == output_2);
    assert(output_3 == output_4);
    assert(output_5 == output_6);
    assert(output_7 == output_8);
    assert(output_9 == output_10);
}

#[test]
fn output_output_neq() {
    let output_1 = Output::Coin;
    let output_2 = Output::Contract;
    let output_3 = Output::Change;
    let output_4 = Output::Variable;
    let output_5 = Output::ContractCreated;

    assert(output_1 != output_2);
    assert(output_1 != output_3);
    assert(output_1 != output_4);
    assert(output_1 != output_5);

    assert(output_2 != output_3);
    assert(output_2 != output_4);
    assert(output_2 != output_5);

    assert(output_3 != output_4);
    assert(output_3 != output_5);

    assert(output_4 != output_5);
}
