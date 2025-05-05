library;

use ::address::Address;
use ::alloc::alloc_bytes;
use ::b512::B512;
use ::bytes::Bytes;
use ::convert::{From, Into, TryFrom};
use ::crypto::{message::Message, public_key::PublicKey, signature_error::SignatureError};
use ::hash::*;
use ::registers::error;
use ::result::Result::{self, *};
use ::option::Option::{self, *};
use ::vm::evm::evm_address::EvmAddress;
use ::ops::*;
use ::codec::*;
use ::debug::*;

/// A secp256k1 signature.
pub struct Secp256k1 {
    /// The underlying raw `[u8; 64]` data of the signature.
    bits: [u8; 64],
}

impl Secp256k1 {
    /// Creates a zeroed out instances of a Secp256k1 signature.
    ///
    /// # Returns
    ///
    /// [Secp256k1] - A zero secp256k1 signature.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::Secp256k1;
    ///
    /// fn foo() {
    ///     let new_secp256k1 = Secp256k1::new();
    ///     assert(new_secp256k1.bits()[0] == 0u8);
    ///     assert(new_secp256k1.bits()[63] == 0u8);
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            bits: [0u8; 64],
        }
    }

    /// Returns the underlying raw `[u8; 64]` data of the signature.
    ///
    /// # Returns
    ///
    /// * [[u8; 64]] - The raw data of the signature.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::Secp256k1;
    ///
    /// fn foo() -> {
    ///     let new_secp256k1 = Secp256k1::new();
    ///     assert(new_secp256k1.bits()[0] == 0u8);
    /// }
    /// ```
    pub fn bits(self) -> [u8; 64] {
        self.bits
    }

    /// Recover the public key derived from the private key used to sign a message.
    /// Returns a `Result` to let the caller choose an error handling strategy.
    ///
    /// # Additional Information
    ///
    /// Follows the Secp256k1 elliptical curve.
    ///
    /// # Arguments
    ///
    /// * `message`: [Message] - The signed data.
    ///
    /// # Returns
    ///
    /// * [Result<PublicKey, SignatureError>] - The recovered public key or an error.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::{Message, PublicKey, Secp256k1};
    ///
    /// fn foo() {
    ///     let signature: Secp256k1 = Secp256k1::from((
    ///         0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c,
    ///         0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d
    ///     ));
    ///     let message: Message = Message::from(0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323);
    ///     let public_key: PublicKey = PublicKey::from((
    ///         0xD73A188181464CC84AE267E45041AEF6AB938F278E636AA1D02D3014C1BEF74E,
    ///         0xC44415635160ACFC87A84300EED97928C949A2D958FC0947C535F7539C59AE75
    ///     ));
    ///
    ///     // A recovered public key pair.
    ///     let result_public_key = signature.recover(message);
    ///
    ///     assert(result_public_key.is_ok());
    ///     assert(result_public_key.unwrap() == public_key);
    /// }
    /// ```
    pub fn recover(self, message: Message) -> Result<PublicKey, SignatureError> {
        let public_key_buffer = (b256::zero(), b256::zero());
        let was_error = asm(
            buffer: __addr_of(public_key_buffer),
            sig: __addr_of(self),
            hash: message.bytes().ptr(),
        ) {
            eck1 buffer sig hash;
            err
        };

        // check the $err register to see if the `eck1` opcode succeeded
        if was_error == 1 {
            Err(SignatureError::UnrecoverablePublicKey)
        } else {
            Ok(PublicKey::from(public_key_buffer))
        }
    }

    /// Recover the address derived from the private key used to sign a message.
    /// Returns a `Result` to let the caller choose an error handling strategy.
    ///
    /// # Additional Information
    ///
    /// Follows the Secp256k1 elliptical curve.
    ///
    /// # Arguments
    ///
    /// * `message`: [Message] - The signed data.
    ///
    /// # Returns
    ///
    /// * [Result<Address, SignatureError>] - The recovered Fuel address or an error.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::{Message, Secp256k1};
    ///
    /// fn foo() {
    ///     let address = Address::from(0x7AAE2D980BE4C3275C72CE5B527FA23FFB97B766966559DD062E2B78FD9D3766);
    ///     let signature: Secp256k1 = Secp256k1::from((
    ///         0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c,
    ///         0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d
    ///     ));
    ///     let message: Message = Message::from(0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323);
    ///
    ///     // A recovered Fuel address.
    ///     let result_address = signature.address(message);
    ///
    ///     assert(result_address.is_ok());
    ///     assert(result_address.unwrap() == address);
    /// }
    /// ```
    pub fn address(self, message: Message) -> Result<Address, SignatureError> {
        match self.recover(message) {
            Ok(pub_key) => Ok(Address::from(sha256(pub_key))),
            Err(e) => Err(e),
        }
    }

    /// Recover the EVM address derived from the private key used to sign a message.
    /// Returns a `Result` to let the caller choose an error handling strategy.
    ///
    /// # Additional Information
    ///
    /// Follows the Secp256k1 elliptical curve.
    ///
    /// # Arguments
    ///
    /// * `message`: [Message] - The signed data.
    ///
    /// # Returns
    ///
    /// * [Result<EvmAddress, SignatureError>] - The recovered evm address or an error.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{vm::evm::evm_address::EvmAddress, crypto::{Secp256k1, Message}};
    ///
    /// fn foo() {
    ///     let evm_address = EvmAddress::from(0x7AAE2D980BE4C3275C72CE5B527FA23FFB97B766966559DD062E2B78FD9D3766);
    ///     let signature: Secp256k1 = Secp256k1::from((
    ///         0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c,
    ///         0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d
    ///     ));
    ///     let message: Message = Message::from(0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323);
    ///     // A recovered evm address.
    ///     let result_address = signature.evm_address(message).unwrap();
    ///     assert(result_address.is_ok());
    ///     assert(result_address.unwrap() == evm_address);
    /// }
    /// ```
    pub fn evm_address(self, message: Message) -> Result<EvmAddress, SignatureError> {
        match self.recover(message) {
            // EVM addresses are derived from the Keccak256 hash of the pubkey, not sha256.
            Ok(pub_key) => Ok(EvmAddress::from(keccak256(pub_key))),
            Err(e) => Err(e),
        }
    }

    /// Verify that a signature matches given public key.
    ///
    /// # Arguments
    ///
    /// * `public_key`: [PublicKey] - The public key to verify against.
    /// * `message`: Message - The signed data.
    ///
    /// # Returns
    ///
    /// * [Result<(), SignatureError>] - An Ok result or an error.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::{Message, PublicKey, Secp256k1};
    ///
    /// fn foo() {
    ///     let signature: Secp256k1 = Secp256k1::from((
    ///         0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c,
    ///         0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d
    ///     ));
    ///     let message: Message = Message::from(0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323);
    ///     let public_key: PublicKey = PublicKey::from((
    ///         0xD73A188181464CC84AE267E45041AEF6AB938F278E636AA1D02D3014C1BEF74E,
    ///         0xC44415635160ACFC87A84300EED97928C949A2D958FC0947C535F7539C59AE75
    ///     ));
    ///
    ///     // A valid result
    ///     let result = signature.verify(public_key, message);
    ///     assert(result.is_ok());
    /// }
    /// ```
    pub fn verify(self, public_key: PublicKey, message: Message) -> Result<(), SignatureError> {
        if public_key.len() != 64 {
            return Err(SignatureError::InvalidPublicKey);
        }

        match self.recover(message) {
            Ok(recovered_pub_key) => if recovered_pub_key == public_key {
                Ok(())
            } else {
                Err(SignatureError::InvalidSignature)
            },
            Err(e) => Err(e),
        }
    }

    /// Verify that an address matches given public key.
    ///
    /// # Arguments
    ///
    /// * `address`: [Address] - The address to verify against.
    /// * `message`: Message - The signed data.
    ///
    /// # Returns
    ///
    /// * [Result<(), SignatureError>] - An Ok result or an error.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::{Message, Secp256k1};
    ///
    /// fn foo() {
    ///     let signature: Secp256k1 = Secp256k1::from((
    ///         0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c,
    ///         0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d
    ///     ));
    ///     let message: Message = Message::from(0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323);
    ///     let address = Address::from(0xD73A188181464CC84AE267E45041AEF6AB938F278E636AA1D02D3014C1BEF74E);
    ///
    ///     // A valid result
    ///     let result = signature.verify_address(address, message);
    ///     assert(result.is_ok());
    /// }
    /// ```
    pub fn verify_address(self, address: Address, message: Message) -> Result<(), SignatureError> {
        match self.address(message) {
            Ok(recovered_address) => if recovered_address == address {
                Ok(())
            } else {
                Err(SignatureError::InvalidSignature)
            },
            Err(e) => Err(e),
        }
    }

    /// Verify that an EVM address matches given public key.
    ///
    /// # Arguments
    ///
    /// * `evm_address`: [EvmAddress] - The EVM address to verify against.
    /// * `message`: Message - The signed data.
    ///
    /// # Returns
    ///
    /// * [Result<(), SignatureError>] - An Ok result or an error.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{crypto::{Message, Secp256k1}, vm::evm::evm_address::EvmAddress};
    ///
    /// fn foo() {
    ///     let signature: Secp256k1 = Secp256k1::from((
    ///         0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c,
    ///         0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d
    ///     ));
    ///     let message: Message = Message::from(0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323);
    ///     let evm_address = EvmAddress::from(0xD73A188181464CC84AE267E45041AEF6AB938F278E636AA1D02D3014C1BEF74E);
    ///
    ///     // A valid result
    ///     let result = signature.verify_evm_address(evm_address, message);
    ///     assert(result.is_ok());
    /// }
    /// ```
    pub fn verify_evm_address(
        self,
        evm_address: EvmAddress,
        message: Message,
) -> Result<(), SignatureError> {
        match self.evm_address(message) {
            Ok(recovered_evm_address) => if recovered_evm_address == evm_address {
                Ok(())
            } else {
                Err(SignatureError::InvalidSignature)
            },
            Err(e) => Err(e),
        }
    }
}

impl From<B512> for Secp256k1 {
    fn from(bits: B512) -> Self {
        Self {
            bits: asm(bits: bits.bits()) {
                bits: [u8; 64]
            },
        }
    }
}

impl From<(b256, b256)> for Secp256k1 {
    fn from(components: (b256, b256)) -> Self {
        Self {
            bits: asm(components: components) {
                components: [u8; 64]
            },
        }
    }
}

impl From<[u8; 64]> for Secp256k1 {
    fn from(array: [u8; 64]) -> Self {
        Self { bits: array }
    }
}

impl TryFrom<Bytes> for Secp256k1 {
    fn try_from(bytes: Bytes) -> Option<Self> {
        if bytes.len() != 64 {
            return None;
        }

        let bits = asm(ptr: bytes.ptr()) {
            ptr: [u8; 64]
        };

        Some(Self { bits })
    }
}

impl Into<B512> for Secp256k1 {
    fn into(self) -> B512 {
        let ptr = __addr_of(self.bits);
        let b256_1 = asm(bits: ptr) {
            bits: b256
        };
        let b256_2 = asm(bits: ptr.add_uint_offset(32)) {
            bits: b256
        };
        B512::from((b256_1, b256_2))
    }
}

impl Into<(b256, b256)> for Secp256k1 {
    fn into(self) -> (b256, b256) {
        let ptr = __addr_of(self.bits);
        let b256_1 = asm(bits: ptr) {
            bits: b256
        };
        let b256_2 = asm(bits: ptr.add_uint_offset(32)) {
            bits: b256
        };
        (b256_1, b256_2)
    }
}

impl Into<Bytes> for Secp256k1 {
    fn into(self) -> Bytes {
        Bytes::from(raw_slice::from_parts::<u8>(__addr_of(self.bits), 64))
    }
}

impl PartialEq for Secp256k1 {
    fn eq(self, other: Self) -> bool {
        asm(result, r2: self.bits, r3: other.bits, r4: 64) {
            meq result r2 r3 r4;
            result: bool
        }
    }
}
impl Eq for Secp256k1 {}

impl Hash for Secp256k1 {
    fn hash(self, ref mut state: Hasher) {
        state.write(Bytes::from(raw_slice::from_parts::<u8>(__addr_of(self.bits), 64)));
    }
}
