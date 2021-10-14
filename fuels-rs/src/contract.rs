use crate::abi_encoder::ABIEncoder;
use crate::errors::Error;
use serde::{Deserialize, Serialize};

use crate::tokens::{Detokenize, Tokenize};
use crate::types::{Function, Selector};

use std::marker::PhantomData;

// TODO: Continue from here
// - [] Refactor all namings now that we have the workflow laid out;
// - [] Keep the call stuff `unimplemented()` for now, focus on abigen-related stuff
// - [] Make sure everything related to code generation is working for all fuel types
// - [] Make `abigen!` work properly, right now it's hardcoded

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
pub struct Call<D> {
    /// The raw transaction object
    pub tx: TransactionRequest, // Maybe not necessary?
    /// The ABI of the function being called
    pub function: Option<Function>, // Temporarily an option
    // To be used in the future:
    // pub block: Option<BlockId>,
    // pub(crate) client: Arc<M>,
    pub datatype: PhantomData<D>,
}

impl<D> Call<D>
where
    D: Detokenize,
{
    pub fn call(&self) -> Result<D, Error> {
        unimplemented!()
    }
}

// TODO: rethink naming
pub struct ContractCall {}

impl ContractCall {
    pub fn new() -> Self {
        Self {}
    }

    // TODO: rethink naming
    // The idea here is that this will just build the request
    pub fn method_hash<T: Tokenize, D: Detokenize>(
        signature: Selector,
        args: T,
    ) -> Result<Call<D>, Error> {
        let mut encoder = ABIEncoder::new();

        let encoded_params = hex::encode(encoder.encode(&args.into_tokens()).unwrap());
        let encoded_selector = hex::encode(signature);

        println!("encoded: {}{}\n", encoded_selector, encoded_params);
        // TODO: In the near future, the actual contract call will happen somewhere here.
        // Right now we're just generating the type-safe bindings with this `method_hash`
        // injected in these bindings.

        let tx = TransactionRequest { data: None };
        Ok(Call {
            tx,
            function: None,
            datatype: PhantomData,
        })
    }
}
