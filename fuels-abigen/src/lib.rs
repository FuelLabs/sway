use fuels_rs::contract::Abigen;
use fuels_rs::contract::Contract;
use fuels_rs::types::ParamType;
use proc_macro::TokenStream;
use syn::parse::Error;

#[proc_macro]
pub fn abigen(input: TokenStream) -> TokenStream {
    println!("input: {:?}\n", input);
    // TODO: continue from here (after working on `contract.rs`).
    // Find a way to turn the `input` into something useful
    // like ethers-rs. Right now this work as-is, hardcoded
    // Does this even need to be a macro...? It does.
    let contract = r#"
    [
        {
            "type":"contract",
            "inputs":[
                {
                    "name":"arg",
                    "type":"u32"
                }
            ],
            "name":"takes_u32_returns_bool",
            "outputs":[
                {
                    "name":"",
                    "type":"bool"
                }
            ]
        }
    ]
    "#;

    let c = Contract::new("test", contract).unwrap();

    c.expand().unwrap().into()
}

/// Contract procedural macro arguments.
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub(crate) struct ContractArgs {
    name: String,
    abi: String,
    parameters: Vec<ParamType>,
}

impl ContractArgs {
    fn into_builder(self) -> Result<Abigen, Error> {
        let mut builder = Abigen::new(&self.name, &self.abi).unwrap();

        // for parameter in self.parameters.into_iter() {
        //     builder = match parameter {
        //         Parameter::Methods(methods) => methods.into_iter().fold(builder, |builder, m| {
        //             builder.add_method_alias(m.signature, m.alias)
        //         }),
        //         Parameter::EventDerives(derives) => derives
        //             .into_iter()
        //             .fold(builder, |builder, derive| builder.add_event_derive(derive)),
        //     };
        // }

        Ok(builder)
    }
}
