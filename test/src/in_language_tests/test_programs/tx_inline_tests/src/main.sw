library;

use std::tx::Transaction;

#[test]
fn tx_transaction_eq() {
    let transaction_1 = Transaction::Script;
    let transaction_2 = Transaction::Script;
    let transaction_3 = Transaction::Create;
    let transaction_4 = Transaction::Create;
    let transaction_5 = Transaction::Mint;
    let transaction_6 = Transaction::Mint;
    let transaction_7 = Transaction::Upgrade;
    let transaction_8 = Transaction::Upgrade;
    let transaction_9 = Transaction::Upload;
    let transaction_10 = Transaction::Upload;
    let transaction_11 = Transaction::Blob;
    let transaction_12 = Transaction::Blob;

    assert(transaction_1 == transaction_1);
    assert(transaction_1 == transaction_2);
    assert(transaction_2 == transaction_2);

    assert(transaction_3 == transaction_3);
    assert(transaction_3 == transaction_4);
    assert(transaction_4 == transaction_4);

    assert(transaction_5 == transaction_5);
    assert(transaction_5 == transaction_6);
    assert(transaction_6 == transaction_6);

    assert(transaction_7 == transaction_7);
    assert(transaction_7 == transaction_8);
    assert(transaction_8 == transaction_8);

    assert(transaction_9 == transaction_9);
    assert(transaction_9 == transaction_10);
    assert(transaction_10 == transaction_10);

    assert(transaction_11 == transaction_11);
    assert(transaction_11 == transaction_12);
    assert(transaction_12 == transaction_12);
}

#[test]
fn tx_transaction_ne() {
    let transaction_1 = Transaction::Script;
    let transaction_2 = Transaction::Script;
    let transaction_3 = Transaction::Create;
    let transaction_4 = Transaction::Create;
    let transaction_5 = Transaction::Mint;
    let transaction_6 = Transaction::Mint;
    let transaction_7 = Transaction::Upgrade;
    let transaction_8 = Transaction::Upgrade;
    let transaction_9 = Transaction::Upload;
    let transaction_10 = Transaction::Upload;
    let transaction_11 = Transaction::Blob;
    let transaction_12 = Transaction::Blob;

    assert(transaction_1 != transaction_3);
    assert(transaction_1 != transaction_4);
    assert(transaction_1 != transaction_5);
    assert(transaction_1 != transaction_6);
    assert(transaction_1 != transaction_7);
    assert(transaction_1 != transaction_8);
    assert(transaction_1 != transaction_9);
    assert(transaction_1 != transaction_10);
    assert(transaction_1 != transaction_11);
    assert(transaction_1 != transaction_12);

    assert(transaction_2 != transaction_3);
    assert(transaction_2 != transaction_4);
    assert(transaction_2 != transaction_5);
    assert(transaction_2 != transaction_6);
    assert(transaction_2 != transaction_7);
    assert(transaction_2 != transaction_8);
    assert(transaction_2 != transaction_9);
    assert(transaction_2 != transaction_10);
    assert(transaction_2 != transaction_11);
    assert(transaction_2 != transaction_12);

    assert(transaction_3 != transaction_5);
    assert(transaction_3 != transaction_6);
    assert(transaction_3 != transaction_7);
    assert(transaction_3 != transaction_8);
    assert(transaction_3 != transaction_9);
    assert(transaction_3 != transaction_10);
    assert(transaction_3 != transaction_11);
    assert(transaction_3 != transaction_12);

    assert(transaction_4 != transaction_5);
    assert(transaction_4 != transaction_6);
    assert(transaction_4 != transaction_7);
    assert(transaction_4 != transaction_8);
    assert(transaction_4 != transaction_9);
    assert(transaction_4 != transaction_10);
    assert(transaction_4 != transaction_11);
    assert(transaction_4 != transaction_12);

    assert(transaction_5 != transaction_7);
    assert(transaction_5 != transaction_8);
    assert(transaction_5 != transaction_9);
    assert(transaction_5 != transaction_10);
    assert(transaction_5 != transaction_11);
    assert(transaction_5 != transaction_12);

    assert(transaction_6 != transaction_7);
    assert(transaction_6 != transaction_8);
    assert(transaction_6 != transaction_9);
    assert(transaction_6 != transaction_10);
    assert(transaction_6 != transaction_11);
    assert(transaction_6 != transaction_12);

    assert(transaction_7 != transaction_9);
    assert(transaction_7 != transaction_10);
    assert(transaction_7 != transaction_11);
    assert(transaction_7 != transaction_12);

    assert(transaction_8 != transaction_9);
    assert(transaction_8 != transaction_10);
    assert(transaction_8 != transaction_11);
    assert(transaction_8 != transaction_12);

    assert(transaction_9 != transaction_11);
    assert(transaction_9 != transaction_12);

    assert(transaction_10 != transaction_11);
    assert(transaction_10 != transaction_12);
}
