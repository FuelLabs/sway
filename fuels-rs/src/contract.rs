use crate::abi_encoder::ABIEncoder;
use crate::errors::Error;
use serde::{Deserialize, Serialize};

use crate::tokens::{Detokenize, Token};
use crate::types::{Function, Selector};

use std::marker::PhantomData;

// Note: This file is a WIP scaffold for a future implementation of the actual
// contract calling infrastructure.

pub struct Contract {}

impl Contract {
    pub fn new() -> Self {
        Self {}
    }

    // The idea here is that this will just build the request
    pub fn method_hash<D: Detokenize>(
        signature: Selector,
        args: &[Token],
    ) -> Result<ContractCall<D>, Error> {
        let mut encoder = ABIEncoder::new();

        let encoded_params = hex::encode(encoder.encode(args).unwrap());
        let encoded_selector = hex::encode(signature);

        // Temporarily printing the encoded selector+params to stdout for
        // debugging purposes.
        println!("encoded: {}{}\n", encoded_selector, encoded_params);

        // TODO: In the near future, the actual contract call will happen somewhere here.
        // Right now we're just generating the type-safe bindings with this `method_hash`
        // injected in these bindings.

        let tx = TransactionRequest { data: None };
        Ok(ContractCall {
            encoded_params,
            encoded_selector,
            tx,
            function: None,
            datatype: PhantomData,
        })
    }
}

/// Parameters for sending a transaction
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct TransactionRequest {
    /// The compiled code of a contract OR the first 4 bytes of the hash of the
    /// invoked method signature and encoded parameters. For details see Ethereum Contract ABI
    pub data: Option<Vec<u8>>,
    // More later
}

#[derive(Debug, Clone)]
#[must_use = "contract calls do nothing unless you `send` or `call` them"]
/// Helper for managing a transaction before submitting it to a node
pub struct ContractCall<D> {
    /// The raw transaction object
    pub tx: TransactionRequest, // Maybe not necessary?
    /// The ABI of the function being called
    pub function: Option<Function>, // Temporarily an option
    // To be used in the future:
    // pub block: Option<BlockId>,
    // pub(crate) client: Arc<M>,
    pub datatype: PhantomData<D>,

    pub encoded_params: String,
    pub encoded_selector: String,
}

impl<D> ContractCall<D>
where
    D: Detokenize,
{
    pub fn call(&self) -> Result<D, Error> {
        unimplemented!()
    }
}
