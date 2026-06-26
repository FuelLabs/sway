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

    assert_eq(transaction_1, transaction_1);
    assert_eq(transaction_1, transaction_2);
    assert_eq(transaction_2, transaction_2);

    assert_eq(transaction_3, transaction_3);
    assert_eq(transaction_3, transaction_4);
    assert_eq(transaction_4, transaction_4);

    assert_eq(transaction_5, transaction_5);
    assert_eq(transaction_5, transaction_6);
    assert_eq(transaction_6, transaction_6);

    assert_eq(transaction_7, transaction_7);
    assert_eq(transaction_7, transaction_8);
    assert_eq(transaction_8, transaction_8);

    assert_eq(transaction_9, transaction_9);
    assert_eq(transaction_9, transaction_10);
    assert_eq(transaction_10, transaction_10);

    assert_eq(transaction_11, transaction_11);
    assert_eq(transaction_11, transaction_12);
    assert_eq(transaction_12, transaction_12);
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

    assert_ne(transaction_1, transaction_3);
    assert_ne(transaction_1, transaction_4);
    assert_ne(transaction_1, transaction_5);
    assert_ne(transaction_1, transaction_6);
    assert_ne(transaction_1, transaction_7);
    assert_ne(transaction_1, transaction_8);
    assert_ne(transaction_1, transaction_9);
    assert_ne(transaction_1, transaction_10);
    assert_ne(transaction_1, transaction_11);
    assert_ne(transaction_1, transaction_12);

    assert_ne(transaction_2, transaction_3);
    assert_ne(transaction_2, transaction_4);
    assert_ne(transaction_2, transaction_5);
    assert_ne(transaction_2, transaction_6);
    assert_ne(transaction_2, transaction_7);
    assert_ne(transaction_2, transaction_8);
    assert_ne(transaction_2, transaction_9);
    assert_ne(transaction_2, transaction_10);
    assert_ne(transaction_2, transaction_11);
    assert_ne(transaction_2, transaction_12);

    assert_ne(transaction_3, transaction_5);
    assert_ne(transaction_3, transaction_6);
    assert_ne(transaction_3, transaction_7);
    assert_ne(transaction_3, transaction_8);
    assert_ne(transaction_3, transaction_9);
    assert_ne(transaction_3, transaction_10);
    assert_ne(transaction_3, transaction_11);
    assert_ne(transaction_3, transaction_12);

    assert_ne(transaction_4, transaction_5);
    assert_ne(transaction_4, transaction_6);
    assert_ne(transaction_4, transaction_7);
    assert_ne(transaction_4, transaction_8);
    assert_ne(transaction_4, transaction_9);
    assert_ne(transaction_4, transaction_10);
    assert_ne(transaction_4, transaction_11);
    assert_ne(transaction_4, transaction_12);

    assert_ne(transaction_5, transaction_7);
    assert_ne(transaction_5, transaction_8);
    assert_ne(transaction_5, transaction_9);
    assert_ne(transaction_5, transaction_10);
    assert_ne(transaction_5, transaction_11);
    assert_ne(transaction_5, transaction_12);

    assert_ne(transaction_6, transaction_7);
    assert_ne(transaction_6, transaction_8);
    assert_ne(transaction_6, transaction_9);
    assert_ne(transaction_6, transaction_10);
    assert_ne(transaction_6, transaction_11);
    assert_ne(transaction_6, transaction_12);

    assert_ne(transaction_7, transaction_9);
    assert_ne(transaction_7, transaction_10);
    assert_ne(transaction_7, transaction_11);
    assert_ne(transaction_7, transaction_12);

    assert_ne(transaction_8, transaction_9);
    assert_ne(transaction_8, transaction_10);
    assert_ne(transaction_8, transaction_11);
    assert_ne(transaction_8, transaction_12);

    assert_ne(transaction_9, transaction_11);
    assert_ne(transaction_9, transaction_12);

    assert_ne(transaction_10, transaction_11);
    assert_ne(transaction_10, transaction_12);
}
