// contract;

// use std::vec::*;

// /// The persistent storage of the contract
// storage {
//     /// A map from addresses to u64s
//     map: StorageMap<Address, u64> = StorageMap::<Address, u64> {},
// }

// /// Documentation for our configurable block
// configurable {
//     /// A u8
//     U8: u8 = 8u8,
//     /// An array of u32s
//     ARRAY: [u32; 3] = [253u32, 254u32, 255u32],
// }

// /// Documentation for out TraitInstance type
// pub trait TestInstance {
//     /// Returns a [Vec] of `len` quasi-random elements
//     /// of the type for which the trait is implemented.
//     fn elements(len: u64) -> Vec<Self>;
// }

// // -- TyTraitType

// /// Documentation for our Container trait
// pub trait Container {
//     /// The type of the elements in the container
//     type E;
//     /// Returns an empty container
//     fn empty() -> Self;
//     /// Inserts an element into the container
//     fn insert(ref mut self, elem: Self::E);
//     /// Removes the last element from the container
//     fn pop_last(ref mut self) -> Option<Self::E>;
// }

// /// Implementation of our Container trait for Vec
// impl<T> Container for Vec<T> {
//     /// The type of the elements in the container
//     type E = T;
//     /// Returns an empty container
//     fn empty() -> Vec<T> { Vec::<T>::new() }
//     /// Inserts an element into the container
//     fn insert(ref mut self, x: T) { self.push(x); }
//     /// Removes the last element from the container
//     fn pop_last(ref mut self) -> Option<T> { self.pop() }
// }

// /// Documentation for our MyContract ABI
// abi MyContract {
//     /// Documentation for our test_function function
//     fn test_function() -> bool;
// }

// impl MyContract for Contract {
//     fn test_function() -> bool {
//         true
//     }
// }


//----------------------------- LIBRARY -----------------------------
// library;

// use std::option::Option;

// /// The `Identity` type: either an `Address` or a `ContractId`.
// pub enum Identity {
//     /// An `Address` variant.
//     Address: u32,
//     /// A `ContractId` variant.
//     ContractId: u32,
// }

//----------------------------- PREDICATE -----------------------------
// predicate;

// fn main() -> bool {
//     false
// }

//----------------------------- SCRIPT -----------------------------
// script;

// fn main() {
//     true
// }

//----------------------------- CONTRACT -----------------------------
contract;

abi MyContract {
    fn test_function() -> bool;
}

impl MyContract for Contract {
    fn test_function() -> bool {
        true
    }
}