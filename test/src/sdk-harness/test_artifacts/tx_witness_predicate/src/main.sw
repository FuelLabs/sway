predicate;

use std::tx::{tx_witnesses_count, tx_witness_data_length, tx_witness_data};

fn main(index: u64, expected_count: u64, expected_length: u64, expected_data: [u8; 64]) -> bool {
    let count: u64 = tx_witnesses_count();
    let length: Option<u64> = tx_witness_data_length(index);
    let data: Option<[u8; 64]> = tx_witness_data(index);

    assert(count == expected_count);
    assert(length.is_some() && length.unwrap() == expected_length);

    assert(data.is_some());
    let data = data.unwrap();    
    let mut iter = 0;
    while iter < 64 {
        assert(data[iter] == expected_data[iter]);
        iter += 1;
    }

    true
}
