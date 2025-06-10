//! The `AssetId` type used for interacting with an asset on the fuel network.
library;

use ::alias::SubId;
use ::contract_id::ContractId;
use ::convert::{From, Into, TryFrom};
use ::block::chain_id;
use ::result::Result::{self, *};
use ::hash::{Hash, Hasher};
use ::ops::*;
use ::primitives::*;
use ::bytes::Bytes;
use ::option::Option::{self, *};
use ::codec::*;
use ::debug::*;
/// An AssetId is used for interacting with an asset on the network.
///
/// # Additional Information
///
/// It is calculated by taking the sha256 hash of the originating ContractId and a SubId.
/// i.e. sha256((contract_id, sub_id)).
///
/// An exception is the Base Asset.
///
/// The SubId is used to differentiate between different assets that are created by the same contract.
pub struct AssetId {
    bits: b256,
}

impl Hash for AssetId {
    fn hash(self, ref mut state: Hasher) {
        let Self { bits } = self;
        bits.hash(state);
    }
}

impl PartialEq for AssetId {
    fn eq(self, other: Self) -> bool {
        self.bits == other.bits
    }
}
impl Eq for AssetId {}

impl From<b256> for AssetId {
    /// Casts raw `b256` data to an `AssetId`.
    ///
    /// # Arguments
    ///
    /// * `bits`: [b256] - The raw `b256` data to be casted.
    ///
    /// # Returns
    ///
    /// * [AssetId] - The newly created `AssetId` from the raw `b256`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///    let asset_id = AssetId::from(b256::zero());
    /// }
    /// ```
    fn from(bits: b256) -> Self {
        Self { bits }
    }
}

pub enum AssetError {
    InvalidChainId: (),
}

impl AssetId {
    /// Creates a new AssetId from a ContractId and SubId.
    ///
    /// # Arguments
    ///
    /// * `contract_id`: [ContractId] - The ContractId of the contract that created the asset.
    /// * `sub_id`: [SubId] - The SubId of the asset.
    ///
    /// # Returns
    ///
    /// * [AssetId] - The AssetId of the asset. Computed by hashing the ContractId and SubId.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::call_frames::contract_id;
    ///
    /// fn foo() {
    ///     let contract_id = contract_id();
    ///     let sub_id = b256::zero();
    ///
    ///     let asset_id = AssetId::new(contract_id, sub_id);
    /// }
    /// ```
    pub fn new(contract_id: ContractId, sub_id: SubId) -> Self {
        let result_buffer = b256::zero();
        asm(
            asset_id: result_buffer,
            ptr: (contract_id, sub_id),
            bytes: 64,
        ) {
            s256 asset_id ptr bytes;
        };

        Self {
            bits: result_buffer,
        }
    }

    /// Creates a new AssetId with the default SubId for the current contract.
    ///
    /// # Additional Information
    ///
    /// **WARNING** If called in an external context, this will **not** return a correct AssetId.
    /// If called externally, will actually use the Transaction Id as the ContractId.
    ///
    /// # Returns
    ///
    /// * [AssetId] - The AssetId of the asset. Computed by hashing the ContractId and the default SubId.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{call_frames::contract_id, constants::DEFAULT_SUB_ID};
    ///
    /// fn foo() {
    ///     let asset_id = AssetId::default();
    ///     assert(asset_id == AssetId::new(contract_id(), DEFAULT_SUB_ID));
    /// }
    /// ```
    pub fn default() -> Self {
        let contract_id = asm() {
            fp: b256
        };
        let result_buffer = b256::zero();
        asm(
            asset_id: result_buffer,
            ptr: (contract_id, b256::zero()),
            bytes: 64,
        ) {
            s256 asset_id ptr bytes;
        };

        Self {
            bits: result_buffer,
        }
    }

    /// The base asset of a chain.
    ///
    /// # Additional Information
    ///
    /// On the Fuel network, the base asset is Ether.
    ///
    /// # Returns
    ///
    /// * [AssetId] - The AssetId of the base asset.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::asset::transfer;
    ///
    /// fn foo() {
    ///     let asset_id = AssetId::base();
    ///     let amount = 100;
    ///     let recipient = Identity::ContractId(ContractId::zero());
    ///
    ///     transfer(recipient, asset_id, amount);
    /// ```
    pub fn base() -> Self {
        Self {
            bits: asm(r1) {
                gm r1 i6;
                r1: b256
            },
        }
    }

    /// Returns the FUEL asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let FUEL_asset = AssetId::fuel();
    ///     // AssetId on mainnet.
    ///     assert(FUEL_asset.bits() == 0x1d5d97005e41cae2187a895fd8eab0506111e0e2f3331cd3912c15c24e3c1d82);
    /// }
    /// ```
    pub fn fuel() -> Result<Self, AssetError> {
        match chain_id() {
            0 => Ok(Self::from(0x324d0c35a4299ef88138a656d5272c5a3a9ccde2630ae055dacaf9d13443d53b)),
            9889 => Ok(Self::from(0x1d5d97005e41cae2187a895fd8eab0506111e0e2f3331cd3912c15c24e3c1d82)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the USDC asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let USDC_asset = AssetId::usdc();
    ///     // AssetId on mainnet.
    ///     assert(USDC_asset.bits() == 0x286c479da40dc953bddc3bb4c453b608bba2e0ac483b077bd475174115395e6b);
    /// }
    /// ```
    pub fn usdc() -> Result<Self, AssetError> {
        match chain_id() {
            0 => Ok(Self::from(0xc26c91055de37528492e7e97d91c6f4abe34aae26f2c4d25cff6bfe45b5dc9a9)),
            9889 => Ok(Self::from(0x286c479da40dc953bddc3bb4c453b608bba2e0ac483b077bd475174115395e6b)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the USDe asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let USDe_asset = AssetId::usde();
    ///     // AssetId on mainnet.
    ///     assert(USDe_asset.bits() == 0xb6133b2ef9f6153eb869125d23dcf20d1e735331b5e41b15a6a7a6cec70e8651);
    /// }
    /// ```
    pub fn usde() -> Result<Self, AssetError> {
        match chain_id() {
            0 => Ok(Self::from(0x86a1beb50c844f5eff9afd21af514a13327c93f76edb89333af862f70040b107)),
            9889 => Ok(Self::from(0xb6133b2ef9f6153eb869125d23dcf20d1e735331b5e41b15a6a7a6cec70e8651)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the sUSDe asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let sUSDe_asset = AssetId::susde();
    ///     // AssetId on mainnet.
    ///     assert(sUSDe_asset.bits() == 0xd05563025104fc36496c15c7021ad6b31034b0e89a356f4f818045d1f48808bc);
    /// }
    /// ```
    pub fn susde() -> Result<Self, AssetError> {
        match chain_id() {
            0 => Ok(Self::from(0xd2886b34454e2e0de47a82d8e6314b26e1e1312519247e8e2ef137672a909aeb)),
            9889 => Ok(Self::from(0xd05563025104fc36496c15c7021ad6b31034b0e89a356f4f818045d1f48808bc)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the wstETH asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let wstETH_asset = AssetId::wsteth();
    ///     // AssetId on mainnet.
    ///     assert(wstETH_asset.bits() == 0x1a7815cc9f75db5c24a5b0814bfb706bb9fe485333e98254015de8f48f84c67b);
    /// }
    /// ```
    pub fn wsteth() -> Result<Self, AssetError> {
        match chain_id() {
            0 => Ok(Self::from(0xb42cd9ddf61898da1701adb3a003b0cf4ca6df7b5fe490ec2c295b1ca43b33c8)),
            9889 => Ok(Self::from(0x1a7815cc9f75db5c24a5b0814bfb706bb9fe485333e98254015de8f48f84c67b)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the WETH asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let WETH_asset = AssetId::weth();
    ///     // AssetId on mainnet.
    ///     assert(WETH_asset.bits() == 0xa38a5a8beeb08d95744bc7f58528073f4052b254def59eba20c99c202b5acaa3);
    /// }
    /// ```
    pub fn weth() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0xa38a5a8beeb08d95744bc7f58528073f4052b254def59eba20c99c202b5acaa3)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the USDT asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let USDT_asset = AssetId::usdt();
    ///     // AssetId on mainnet.
    ///     assert(USDT_asset.bits() == 0xa0265fb5c32f6e8db3197af3c7eb05c48ae373605b8165b6f4a51c5b0ba4812e);
    /// }
    /// ```
    pub fn usdt() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0xa0265fb5c32f6e8db3197af3c7eb05c48ae373605b8165b6f4a51c5b0ba4812e)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the weEth asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let weEth_asset = AssetId::weeth();
    ///     // AssetId on mainnet.
    ///     assert(weEth_asset.bits() == 0x239ed6e12b7ce4089ee245244e3bf906999a6429c2a9a445a1e1faf56914a4ab);
    /// }
    /// ```
    pub fn weeth() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0x239ed6e12b7ce4089ee245244e3bf906999a6429c2a9a445a1e1faf56914a4ab)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the rsETH asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let rsEth_asset = AssetId::rseth();
    ///     // AssetId on mainnet.
    ///     assert(rsEth_asset.bits() == 0xbae80f7fb8aa6b90d9b01ef726ec847cc4f59419c4d5f2ea88fec785d1b0e849);
    /// }
    /// ```
    pub fn rseth() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0xbae80f7fb8aa6b90d9b01ef726ec847cc4f59419c4d5f2ea88fec785d1b0e849)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the rETH asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let rEth_asset = AssetId::reth();
    ///     // AssetId on mainnet.
    ///     assert(rEth_asset.bits() == 0xf3f9a0ed0ce8eac5f89d6b83e41b3848212d5b5f56108c54a205bb228ca30c16);
    /// }
    /// ```
    pub fn reth() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0xf3f9a0ed0ce8eac5f89d6b83e41b3848212d5b5f56108c54a205bb228ca30c16)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the wbETH asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let wbETH_asset = AssetId::wbeth();
    ///     // AssetId on mainnet.
    ///     assert(wbETH_asset.bits() == 0x7843c74bef935e837f2bcf67b5d64ecb46dd53ff86375530b0caf3699e8ffafe);
    /// }
    /// ```
    pub fn wbeth() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0x7843c74bef935e837f2bcf67b5d64ecb46dd53ff86375530b0caf3699e8ffafe)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the rstETH asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let rstETH_asset = AssetId::rsteth();
    ///     // AssetId on mainnet.
    ///     assert(rstETH_asset.bits() == 0x962792286fbc9b1d5860b4551362a12249362c21594c77abf4b3fe2bbe8d977a);
    /// }
    /// ```
    pub fn rsteth() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0x962792286fbc9b1d5860b4551362a12249362c21594c77abf4b3fe2bbe8d977a)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the amphrETH asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let amphrETH_asset = AssetId::amphreth();
    ///     // AssetId on mainnet.
    ///     assert(amphrETH_asset.bits() == 0x05fc623e57bd7bc1258efa8e4f62b05af5471d73df6f2c2dc11ecc81134c4f36);
    /// }
    /// ```
    pub fn amphreth() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0x05fc623e57bd7bc1258efa8e4f62b05af5471d73df6f2c2dc11ecc81134c4f36)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the Manta mBTC asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let manta_mBTC_asset = AssetId::manta_mbtc();
    ///     // AssetId on mainnet.
    ///     assert(manta_mBTC_asset.bits() == 0xaf3111a248ff7a3238cdeea845bb2d43cf3835f1f6b8c9d28360728b55b9ce5b);
    /// }
    /// ```
    pub fn manta_mbtc() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0xaf3111a248ff7a3238cdeea845bb2d43cf3835f1f6b8c9d28360728b55b9ce5b)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the Manta mETH asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let manta_mETH_asset = AssetId::manta_meth();
    ///     // AssetId on mainnet.
    ///     assert(manta_mETH_asset.bits() == 0xafd219f513317b1750783c6581f55530d6cf189a5863fd18bd1b3ffcec1714b4);
    /// }
    /// ```
    pub fn manta_meth() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0xafd219f513317b1750783c6581f55530d6cf189a5863fd18bd1b3ffcec1714b4)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the Manta mUSD asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let manta_mUSD_asset = AssetId::manta_musd();
    ///     // AssetId on mainnet.
    ///     assert(manta_mUSD_asset.bits() == 0x89cb9401e55d49c3269654dd1cdfb0e80e57823a4a7db98ba8fc5953b120fef4);
    /// }
    /// ```
    pub fn manta_musd() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0x89cb9401e55d49c3269654dd1cdfb0e80e57823a4a7db98ba8fc5953b120fef4)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the pumpBTC asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let pumpBTC_asset = AssetId::pumpbtc();
    ///     // AssetId on mainnet.
    ///     assert(pumpBTC_asset.bits() == 0x0aa5eb2bb97ca915288b653a2529355d4dc66de2b37533213f0e4aeee3d3421f);
    /// }
    /// ```
    pub fn pumpbtc() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0x0aa5eb2bb97ca915288b653a2529355d4dc66de2b37533213f0e4aeee3d3421f)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the FBTC asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let FBTC_asset = AssetId::fbtc();
    ///     // AssetId on mainnet.
    ///     assert(FBTC_asset.bits() == 0xb5ecb0a1e08e2abbabf624ffea089df933376855f468ade35c6375b00c33996a);
    /// }
    /// ```
    pub fn fbtc() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0xb5ecb0a1e08e2abbabf624ffea089df933376855f468ade35c6375b00c33996a)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the SolvBTC asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let solvBTC_asset = AssetId::solvbtc();
    ///     // AssetId on mainnet.
    ///     assert(solvBTC_asset.bits() == 0x1186afea9affb88809c210e13e2330b5258c2cef04bb8fff5eff372b7bd3f40f);
    /// }
    /// ```
    pub fn solvbtc() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0x1186afea9affb88809c210e13e2330b5258c2cef04bb8fff5eff372b7bd3f40f)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the SolvBTC.BBN asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let SolvBTC_BBN_asset = AssetId::solvbtc_bnn();
    ///     // AssetId on mainnet.
    ///     assert(SolvBTC_BBN_asset.bits() == 0x7a4f087c957d30218223c2baaaa365355c9ca81b6ea49004cfb1590a5399216f);
    /// }
    /// ```
    pub fn solvbtc_bnn() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0x7a4f087c957d30218223c2baaaa365355c9ca81b6ea49004cfb1590a5399216f)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the Mantle mETH	 asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let mantle_mETH_asset = AssetId::mantle_meth();
    ///     // AssetId on mainnet.
    ///     assert(mantle_mETH_asset.bits() == 0x642a5db59ec323c2f846d4d4cf3e58d78aff64accf4f8f6455ba0aa3ef000a3b);
    /// }
    /// ```
    pub fn mantle_meth() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0x642a5db59ec323c2f846d4d4cf3e58d78aff64accf4f8f6455ba0aa3ef000a3b)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the sDAI asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let sDAI_asset = AssetId::sdai();
    ///     // AssetId on mainnet.
    ///     assert(sDAI_asset.bits() == 0x9e46f919fbf978f3cad7cd34cca982d5613af63ff8aab6c379e4faa179552958);
    /// }
    /// ```
    pub fn sdai() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0x9e46f919fbf978f3cad7cd34cca982d5613af63ff8aab6c379e4faa179552958)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the rsUSDe asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let rsUSDe_asset = AssetId::rsusde();
    ///     // AssetId on mainnet.
    ///     assert(rsUSDe_asset.bits() == 0x78d4522ec607f6e8efb66ea49439d1ee48623cf763f9688a8eada025def033d9);
    /// }
    /// ```
    pub fn rsusde() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0x78d4522ec607f6e8efb66ea49439d1ee48623cf763f9688a8eada025def033d9)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the ezETH asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let ezETH_asset = AssetId::ezeth();
    ///     // AssetId on mainnet.
    ///     assert(ezETH_asset.bits() == 0x91b3559edb2619cde8ffb2aa7b3c3be97efd794ea46700db7092abeee62281b0);
    /// }
    /// ```
    pub fn ezeth() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0x91b3559edb2619cde8ffb2aa7b3c3be97efd794ea46700db7092abeee62281b0)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the pzETH asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let pzETH_asset = AssetId::pzeth();
    ///     // AssetId on mainnet.
    ///     assert(pzETH_asset.bits() == 0x1493d4ec82124de8f9b625682de69dcccda79e882b89a55a8c737b12de67bd68);
    /// }
    /// ```
    pub fn pzeth() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0x1493d4ec82124de8f9b625682de69dcccda79e882b89a55a8c737b12de67bd68)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the Re7LRT asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let Re7LRT_asset = AssetId::re7lrt();
    ///     // AssetId on mainnet.
    ///     assert(Re7LRT_asset.bits() == 0xf2fc648c23a5db24610a1cf696acc4f0f6d9a7d6028dd9944964ab23f6e35995);
    /// }
    /// ```
    pub fn re7lrt() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0xf2fc648c23a5db24610a1cf696acc4f0f6d9a7d6028dd9944964ab23f6e35995)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the steakLRT asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let steakLRT_asset = AssetId::steaklrt();
    ///     // AssetId on mainnet.
    ///     assert(steakLRT_asset.bits() == 0x4fc8ac9f101df07e2c2dec4a53c8c42c439bdbe5e36ea2d863a61ff60afafc30);
    /// }
    /// ```
    pub fn steaklrt() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0x4fc8ac9f101df07e2c2dec4a53c8c42c439bdbe5e36ea2d863a61ff60afafc30)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the USDF asset.
    ///
    /// # Additional Information
    ///
    /// Verified addresses can be found at https://docs.fuel.network/docs/verified-addresses/assets/.
    ///
    /// # Returns
    ///
    /// * [Result<AssetId, AssetError>] - `Ok(AssetId)` or `Err(AssetError)` if called on an unrecognized chain or the asset has not been verified.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let USDF_asset = AssetId::usdf();
    ///     // AssetId on mainnet.
    ///     assert(USDF_asset.bits() == 0x33a6d90877f12c7954cca6d65587c25e9214c7bed2231c188981c7114c1bdb78);
    /// }
    /// ```
    pub fn usdf() -> Result<Self, AssetError> {
        match chain_id() {
            9889 => Ok(Self::from(0x33a6d90877f12c7954cca6d65587c25e9214c7bed2231c188981c7114c1bdb78)),
            _ => Err(AssetError::InvalidChainId),
        }
    }

    /// Returns the underlying raw `b256` data of the asset id.
    ///
    /// # Returns
    ///
    /// * [b256] - The raw data of the asset id.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() -> {
    ///     let my_asset = AssetId::from(b256::zero());
    ///     assert(my_asset.bits() == b256::zero());
    /// }
    /// ```
    pub fn bits(self) -> b256 {
        self.bits
    }

    /// Returns the zero value for the `AssetId` type.
    ///
    /// # Returns
    ///
    /// * [AssetId] -> The zero value for the `AssetId` type.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_asset_id = AssetId::zero();
    ///     assert(zero_asset_id == AssetId::from(b256::zero()));
    /// }
    /// ```
    pub fn zero() -> Self {
        Self {
            bits: b256::zero(),
        }
    }

    /// Returns whether an `AssetId` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `AssetId` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_asset_id = AssetId::zero();
    ///     assert(zero_asset_id.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self.bits == b256::zero()
    }
}

impl From<AssetId> for b256 {
    /// Casts an `AssetId` to raw `b256` data.
    ///
    /// # Returns
    ///
    /// * [b256] - The underlying raw `b256` data of the `AssetId`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let asset_id = AssetId::zero();
    ///     let b256_data: b256 = b256::from(asset_id);
    ///     assert(b256_data == b256::zero());
    /// }
    /// ```
    fn from(id: AssetId) -> Self {
        id.bits()
    }
}

impl TryFrom<Bytes> for AssetId {
    /// Casts raw `Bytes` data to an `AssetId`.
    ///
    /// # Arguments
    ///
    /// * `bytes`: [Bytes] - The raw `Bytes` data to be casted.
    ///
    /// # Returns
    ///
    /// * [AssetId] - The newly created `AssetId` from the raw `Bytes`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// fn foo(bytes: Bytes) {
    ///    let result = AssetId::try_from(bytes);
    ///    assert(result.is_some());
    ///    let asset_id = result.unwrap();
    /// }
    /// ```
    fn try_from(bytes: Bytes) -> Option<Self> {
        if bytes.len() != 32 {
            return None;
        }

        Some(Self {
            bits: asm(ptr: bytes.ptr()) {
                ptr: b256
            },
        })
    }
}

impl Into<Bytes> for AssetId {
    /// Casts an `AssetId` to raw `Bytes` data.
    ///
    /// # Returns
    ///
    /// * [Bytes] - The underlying raw `Bytes` data of the `AssetId`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let asset_id = AssetId::zero();
    ///     let bytes_data: Bytes = asset_id.into();
    ///     assert(bytes_data.len() == 32);
    /// }
    /// ```
    fn into(self) -> Bytes {
        Bytes::from(self.bits())
    }
}
