# Native Assets

<!-- This section should explain native assets in Sway -->
<!-- native_assets:example:start -->
The FuelVM has built-in support for working with multiple assets.

## Key Differences Between EVM and FuelVM Assets

### ERC-20 vs Native Asset

On the EVM, Ether is the native asset. As such, sending ETH to an address or contract is an operation built into the EVM, meaning it doesn't rely on the existence of a smart contract to update balances to track ownership as with ERC-20 tokens.

On the FuelVM, _all_ assets are native and the process for sending _any_ native asset is the same.

While you would still need a smart contract to handle the minting and burning of assets, the sending and receiving of these assets can be done independently of the asset contract.

Just like the EVM however, Fuel has a standard that describes a standard API for Native Assets using the Sway Language. The ERC-20 equivalent for the Sway Language is the [SRC-20; Native Asset Standard](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-20.md).

> **NOTE** It is important to note that Fuel does not have tokens.

### ERC-721 vs Native Asset

On the EVM, an ERC-721 token or NFT is a contract that contains multiple tokens which are non-fungible with one another.

On the FuelVM, the ERC-721 equivalent is a Native Asset where each asset has a supply of one. This is defined in the [SRC-20; Native Asset Standard](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-20.md#non-fungible-asset-restrictions) under the Non-Fungible Asset Restrictions.

In practice, this means all NFTs are treated the same as any other Native Asset on Fuel. When writing Sway code, no additional cases for handling non-fungible and fungible assets are required.

### No Token Approvals

An advantage Native Assets bring is that there is no need for token approvals; as with Ether on the EVM. With millions of dollars hacked every year due to misused token approvals, the FuelVM eliminates this attack vector.

### Asset vs Coin vs Token

An "Asset" is a Native Asset on Fuel and has the associated `AssetId` type. Assets are distinguishable from one another. A "Coin" represents a singular unit of an Asset. Coins of the same Asset are not distinguishable from one another.

Fuel does not use tokens like other ecosystems such as Ethereum and uses Native Assets with a UTXO design instead.

## The `AssetId` type

The `AssetId` type represents any Native Asset on Fuel. An `AssetId` is used for interacting with an asset on the network.

The `AssetId` of any Native Asset on Fuel is calculated by taking the SHA256 hash digest of the originating `ContractId` that minted the asset and a `SubId` i.e. `sha256((contract_id, sub_id))`.

### Creating a New `AssetId`

There are 3 ways to instantiate a new `AssetId`:

#### Default

When a contract will only ever mint a single asset, it is recommended to use the `DEFAULT_ASSET_ID` sub id. This is referred to as the default asset of a contract.

To get the default asset from an internal contract call, call the `default()` function:

```sway
{{#include ../../../../examples/native_asset/src/main.sw:default_asset_id}}
```

#### New

If a contract mints multiple assets or if the asset has been minted by an external contract, the `new()` function will be needed. The `new()` function takes the `ContractId` of the contract which minted the token as well as a `SubId`.

To create a new `AssetId` using a `ContractId` and `SubId`, call the `new()` function:

```sway
{{#include ../../../../examples/native_asset/src/main.sw:new_asset_id}}
```

#### From

In the case where the `b256` value of an asset is already known, you may call the `from()` function with the `b256` value.

```sway
{{#include ../../../../examples/native_asset/src/main.sw:from_asset_id}}
```

## The `SubId` type

The SubId is used to differentiate between different assets that are created by the same contract. The `SubId` is a `b256` value.

When creating a single new asset on Fuel, we recommend using the `DEFAULT_SUB_ID` or `SubId::zero()`.

## The Base Asset

On the Fuel Network, the base asset is Ether. This is the only asset on the Fuel Network that does not have a `SubId`.

The Base Asset can be returned anytime by calling the `base()` function of the `AssetId` type.

```sway
{{#include ../../../../examples/native_asset/src/main.sw:base_asset}}
```

## Basic Native Asset Functionality

### Minting A Native Asset

To mint a new asset, the `std::asset::mint()` function must be called internally within a contract. A `SubId` and amount of coins must be provided. These newly minted coins will be owned by the contract which minted them. To mint another asset from the same contract, replace the `DEFAULT_SUB_ID` with your desired `SubId`.

```sway
{{#include ../../../../examples/native_asset/src/main.sw:mint_asset}}
```

You may also mint an asset to a specific entity with the `std::asset::mint_to()` function. Be sure to provide a target `Identity` that will own the newly minted coins.

```sway
{{#include ../../../../examples/native_asset/src/main.sw:mint_to_asset}}
```

If you intend to allow external users to mint assets using your contract, the [SRC-3; Mint and Burn Standard](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-3.md#fn-mintrecipient-identity-vault_sub_id-subid-amount-u64) defines a standard API for minting assets. The [Sway-Libs Asset Library](https://fuellabs.github.io/sway-libs/book/asset/supply.html) also provides an additional library to support implementations of the SRC-3 Standard into your contract.

### Burning a Native Asset

To burn an asset, the `std::asset::burn()` function must be called internally from the contract which minted them. The `SubId` used to mint the coins and amount must be provided. The burned coins must be owned by the contract. When an asset is burned it doesn't exist anymore.

```sway
{{#include ../../../../examples/native_asset/src/main.sw:burn_asset}}
```

If you intend to allow external users to burn assets using your contract, the [SRC-3; Mint and Burn Standard](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-3.md#fn-mintrecipient-identity-vault_sub_id-subid-amount-u64) defines a standard API for burning assets. The [Sway-Libs Asset Library](https://fuellabs.github.io/sway-libs/book/asset/supply.html) also provides an additional library to support implementations of the SRC-3 Standard into your contract.

### Transfer a Native Asset

To internally transfer a Native Asset, the `std::asset::transfer()` function must be called. A target `Identity` or user must be provided as well as the `AssetId` of the asset and an amount.

```sway
{{#include ../../../../examples/native_asset/src/main.sw:transfer_asset}}
```

### Native Asset And Transactions

#### Getting The Transaction Asset

To query for the Native Asset sent in a transaction, you may call the `std::call_frames::msg_asset_id()` function.

```sway
{{#include ../../../../examples/native_asset/src/main.sw:msg_asset_id}}
```

#### Getting The Transaction Amount

To query for the amount of coins sent in a transaction, you may call the `std::context::msg_amount()` function.

```sway
{{#include ../../../../examples/native_asset/src/main.sw:msg_amount}}
```

### Native Assets and Contracts

#### Checking A Contract's Balance

To internally check a contract's balance, call the `std::context::this_balance()` function with the corresponding `AssetId`.

```sway
{{#include ../../../../examples/native_asset/src/main.sw:this_balance}}
```

To check the balance of an external contract, call the `std::context::balance_of()` function with the corresponding `AssetId`.

```sway
{{#include ../../../../examples/native_asset/src/main.sw:balance_of}}
```

> **NOTE** Due to the FuelVM's UTXO design, balances of `Address`'s cannot be returned in the Sway Language. This must be done off-chain using the SDK.

#### Receiving Native Assets In A Contract

By default, a contract may not receive a Native Asset in a contract call. To allow transferring of assets to the contract, add the `#[payable]` attribute to the function.

```sway
{{#include ../../../../examples/native_asset/src/main.sw:payable}}
```

## Native Asset Standards

There are a number of standards developed to enable further functionality for Native Assets and help cross contract functionality. Information on standards can be found in the [Sway Standards Repo](https://github.com/FuelLabs/sway-standards).

We currently have the following standards for Native Assets:

- [SRC-20; Native Asset Standard](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-20.md) defines the implementation of a standard API for Native Assets using the Sway Language.
- [SRC-3; Mint and Burn Standard](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-3.md) is used to enable mint and burn functionality for Native Assets.
- [SRC-7; Arbitrary Asset Metadata Standard](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-7.md) is used to store metadata for Native Assets.
- [SRC-6; Vault Standard](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-6.md) defines the implementation of a standard API for asset vaults developed in Sway.

## Native Asset Libraries

Additional Libraries have been developed to allow you to quickly create an deploy dApps that follow the [Sway Standards](https://github.com/FuelLabs/sway-standards).

- [Asset Library](https://fuellabs.github.io/sway-libs/book/asset/index.html) provides functionality to implement the [SRC-20; Native Asset Standard](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-20.md), [SRC-3; Mint and Burn Standard](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-3.md), and [SRC-7; Arbitrary Asset Metadata Standard](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-7.md) standards.

<!-- native_assets:example:end -->

## Single Native Asset Example

In this fully fleshed out example, we show a native asset contract which mints a single asset. This is the equivalent to the ERC-20 Standard use in Ethereum. Note there are no token approval functions.

It implements the [SRC-20; Native Asset](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-20.md), [SRC-3; Mint and Burn](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-3.md), and [SRC-5; Ownership](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-5.md) standards. It does not use any external libraries.

```sway
// ERC20 equivalent in Sway.
contract;

use src3::SRC3;
use src5::{SRC5, State, AccessError};
use src20::SRC20;
use std::{
    asset::{
        burn,
        mint_to,
    },
    call_frames::{
        contract_id,
        msg_asset_id,
    },
    constants::DEFAULT_SUB_ID,
    context::msg_amount,
    string::String,
};

configurable {
    DECIMALS: u8 = 9u8,
    NAME: str[7] = __to_str_array("MyAsset"),
    SYMBOL: str[5] = __to_str_array("MYTKN"),
}

storage {
    total_supply: u64 = 0,
    owner: State = State::Uninitialized,
}

// Native Asset Standard
impl SRC20 for Contract {
    #[storage(read)]
    fn total_assets() -> u64 {
        1
    }

    #[storage(read)]
    fn total_supply(asset: AssetId) -> Option<u64> {
        if asset == AssetId::default() {
            Some(storage.total_supply.read())
        } else {
            None
        }
    }

    #[storage(read)]
    fn name(asset: AssetId) -> Option<String> {
        if asset == AssetId::default() {
            Some(String::from_ascii_str(from_str_array(NAME)))
        } else {
            None
        }
    }

    #[storage(read)]
    fn symbol(asset: AssetId) -> Option<String> {
        if asset == AssetId::default() {
            Some(String::from_ascii_str(from_str_array(SYMBOL)))
        } else {
            None
        }
    }

    #[storage(read)]
    fn decimals(asset: AssetId) -> Option<u8> {
        if asset == AssetId::default() {
            Some(DECIMALS)
        } else {
            None
        }
    }
}

// Ownership Standard
impl SRC5 for Contract {
    #[storage(read)]
    fn owner() -> State {
        storage.owner.read()
    }
}

// Mint and Burn Standard
impl SRC3 for Contract {
    #[storage(read, write)]
    fn mint(recipient: Identity, sub_id: SubId, amount: u64) {
        require(sub_id == DEFAULT_SUB_ID, "incorrect-sub-id");
        require_access_owner();

        storage
            .total_supply
            .write(amount + storage.total_supply.read());
        mint_to(recipient, DEFAULT_SUB_ID, amount);
    }

    #[storage(read, write)]
    fn burn(sub_id: SubId, amount: u64) {
        require(sub_id == DEFAULT_SUB_ID, "incorrect-sub-id");
        require(msg_amount() >= amount, "incorrect-amount-provided");
        require(
            msg_asset_id() == AssetId::default(),
            "incorrect-asset-provided",
        );
        require_access_owner();

        storage
            .total_supply
            .write(storage.total_supply.read() - amount);
        burn(DEFAULT_SUB_ID, amount);
    }
}

abi SingleAsset {
    #[storage(read, write)]
    fn constructor(owner_: Identity);
}

impl SingleAsset for Contract {
    #[storage(read, write)]
    fn constructor(owner_: Identity) {
        require(storage.owner.read() == State::Uninitialized, "owner-initialized");
        storage.owner.write(State::Initialized(owner_));
    }
}

#[storage(read)]
fn require_access_owner() {
    require(
        storage.owner.read() == State::Initialized(msg_sender().unwrap()),
        AccessError::NotOwner,
    );
}
```

## Multi Native Asset Example

In this fully fleshed out example, we show a native asset contract which mints multiple assets. This is the equivalent to the ERC-1155 Standard use in Ethereum. Note there are no token approval functions.

It implements the [SRC-20; Native Asset](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-20.md), [SRC-3; Mint and Burn](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-3.md), and [SRC-5; Ownership](https://github.com/FuelLabs/sway-standards/blob/master/SRCs/src-5.md) standards. It does not use any external libraries.

```sway
// ERC1155 equivalent in Sway.
contract;

use src5::{SRC5, State, AccessError};
use src20::SRC20;
use src3::SRC3;
use std::{
    asset::{
        burn,
        mint_to,
    },
    call_frames::{
        contract_id,
        msg_asset_id,
    },
    hash::{
        Hash,
    },
    context::this_balance,
    storage::storage_string::*,
    string::String
};

storage {
    total_assets: u64 = 0,
    total_supply: StorageMap<AssetId, u64> = StorageMap {},
    name: StorageMap<AssetId, StorageString> = StorageMap {},
    symbol: StorageMap<AssetId, StorageString> = StorageMap {},
    decimals: StorageMap<AssetId, u8> = StorageMap {},
    owner: State = State::Uninitialized,
}

// Native Asset Standard
impl SRC20 for Contract {
    #[storage(read)]
    fn total_assets() -> u64 {
        storage.total_assets.read()
    }

    #[storage(read)]
    fn total_supply(asset: AssetId) -> Option<u64> {
        storage.total_supply.get(asset).try_read()
    }

    #[storage(read)]
    fn name(asset: AssetId) -> Option<String> {
        storage.name.get(asset).read_slice()
    }
    
    #[storage(read)]
    fn symbol(asset: AssetId) -> Option<String> {
        storage.symbol.get(asset).read_slice()
    }

    #[storage(read)]
    fn decimals(asset: AssetId) -> Option<u8> {
        storage.decimals.get(asset).try_read()
    }
}

// Mint and Burn Standard
impl SRC3 for Contract {
    #[storage(read, write)]
    fn mint(recipient: Identity, sub_id: SubId, amount: u64) {
        require_access_owner();
        let asset_id = AssetId::new(contract_id(), sub_id);
        let supply = storage.total_supply.get(asset_id).try_read();
        if supply.is_none() {
            storage.total_assets.write(storage.total_assets.try_read().unwrap_or(0) + 1);
        }
        let current_supply = supply.unwrap_or(0);
        storage.total_supply.insert(asset_id, current_supply + amount);
        mint_to(recipient, sub_id, amount);
    }
    
    #[storage(read, write)]
    fn burn(sub_id: SubId, amount: u64) {
        require_access_owner();
        let asset_id = AssetId::new(contract_id(), sub_id);
        require(this_balance(asset_id) >= amount, "not-enough-coins");
        
        let supply = storage.total_supply.get(asset_id).try_read();
        let current_supply = supply.unwrap_or(0);
        storage.total_supply.insert(asset_id, current_supply - amount);
        burn(sub_id, amount);
    }
}

abi MultiAsset {
    #[storage(read, write)]
    fn constructor(owner_: Identity);
    
    #[storage(read, write)]
    fn set_name(asset: AssetId, name: String);

    #[storage(read, write)]
    fn set_symbol(asset: AssetId, symbol: String);

    #[storage(read, write)]
    fn set_decimals(asset: AssetId, decimals: u8);
}

impl MultiAsset for Contract {
    #[storage(read, write)]
    fn constructor(owner_: Identity) {
        require(storage.owner.read() == State::Uninitialized, "owner-initialized");
        storage.owner.write(State::Initialized(owner_));
    }
    
    #[storage(read, write)]
    fn set_name(asset: AssetId, name: String) {
        require_access_owner();
        storage.name.insert(asset, StorageString {});
        storage.name.get(asset).write_slice(name);
    }

    #[storage(read, write)]
    fn set_symbol(asset: AssetId, symbol: String) {
        require_access_owner();
        storage.symbol.insert(asset, StorageString {});
        storage.symbol.get(asset).write_slice(symbol);
    }

    #[storage(read, write)]
    fn set_decimals(asset: AssetId, decimals: u8) {
        require_access_owner();
        storage.decimals.insert(asset, decimals);
    }
}

#[storage(read)]
fn require_access_owner() {
    require(
        storage.owner.read() == State::Initialized(msg_sender().unwrap()),
        AccessError::NotOwner,
    );
}
```
