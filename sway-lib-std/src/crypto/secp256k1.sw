library;

use ::address::Address;
use ::alloc::alloc_bytes;
use ::b512::B512;
use ::bytes::Bytes;
use ::convert::From;
use ::crypto::{cryptographic_error::CryptographicError, message::Message, public_key::PublicKey};
use ::hash::*;
use ::registers::error;
use ::result::Result::{self, *};
use ::vm::evm::evm_address::EvmAddress;

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
            bits: [
                0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
                0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
                0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
                0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
                0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            ],
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
}

impl Secp256k1 {
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
    /// * [Result<PublicKey, CryptographicError>] - The recovered public key or an error.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::{Message, PublicKey, Secp256k1};
    ///
    /// fn foo() {
    ///     let hi = 0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c;
    ///     let lo = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    ///     let msg_hash = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    ///     let pub_hi = 0xD73A188181464CC84AE267E45041AEF6AB938F278E636AA1D02D3014C1BEF74E;
    ///     let pub_lo = 0xC44415635160ACFC87A84300EED97928C949A2D958FC0947C535F7539C59AE75;
    ///     let signature: Secp256k1 = Secp256k1::from((hi, lo));
    ///     let message: Message = Message::from(msg_hash);
    ///     let public_key: PublicKey = PublicKey::from((pub_hi, pub_lo));
    ///
    ///     // A recovered public key pair.
    ///     let result_public_key = signature.recover(message);
    ///
    ///     assert(result_public_key.is_ok());
    ///     assert(result_public_key.unwrap() == public_key);
    /// }
    /// ```
    pub fn recover(self, message: Message) -> Result<PublicKey, CryptographicError> {
        let public_key = PublicKey::new();
        let was_error = asm(
            buffer: __addr_of(public_key),
            sig: __addr_of(self),
            hash: __addr_of(message),
        ) {
            eck1 buffer sig hash;
            err
        };

        // check the $err register to see if the `eck1` opcode succeeded
        if was_error == 1 {
            Err(CryptographicError::UnrecoverablePublicKey)
        } else {
            Ok(public_key)
        }
    }
}

impl Secp256k1 {
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
    /// * [Result<Address, CryptographicError>] - The recovered Fuel address or an error.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::{Message, Secp256k1};
    ///
    /// fn foo() {
    ///     let hi = 0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c;
    ///     let lo = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    ///     let msg_hash = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    ///     let address = Address::from(0x7AAE2D980BE4C3275C72CE5B527FA23FFB97B766966559DD062E2B78FD9D3766);
    ///     let signature: Secp256k1 = Secp256k1::from((hi, lo));
    ///     let message: Message = Message::from(msg_hash);
    ///
    ///     // A recovered Fuel address.
    ///     let result_address = signature.address(message);
    ///
    ///     assert(result_address.is_ok());
    ///     assert(result_address.unwrap() == address);
    /// }
    /// ```
    pub fn address(self, message: Message) -> Result<Address, CryptographicError> {
        let pub_key_result = Self::recover(self, message);

        if let Err(e) = pub_key_result {
            // propagate the error if it exists
            Err(e)
        } else {
            let pub_key = pub_key_result.unwrap();
            let address = sha256(pub_key);
            Ok(Address::from(address))
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
    /// * [Result<EvmAddress, CryptographicError>] - The recovered evm address or an error.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{vm::evm::evm_address::EvmAddress, crypto::{Secp256k1, Message}};
    ///
    /// fn foo() {
    ///     let hi = 0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c;
    ///     let lo = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    ///     let msg_hash = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    ///     let evm_address = EvmAddress::from(0x7AAE2D980BE4C3275C72CE5B527FA23FFB97B766966559DD062E2B78FD9D3766);
    ///     let signature: Secp256k1 = Secp256k1::from((hi, lo));
    ///     let message: Message = Message::from(msg_hash);
    ///     // A recovered evm address.
    ///     let result_address = signature.evm_address(message).unwrap();
    ///     assert(result_address.is_ok());
    ///     assert(result_address.unwrap() == evm_address);
    /// }
    /// ```
    pub fn evm_address(self, message: Message) -> Result<EvmAddress, CryptographicError> {
        let pub_key_result = Self::recover(self, message);

        if let Err(e) = pub_key_result {
            // propagate the error if it exists
            Err(e)
        } else {
            let pub_key = pub_key_result.unwrap();
            // Note that EVM addresses are derived from the Keccak256 hash of the pubkey (not sha256)
            let evm_address_hash = keccak256(pub_key);
            Ok(EvmAddress::from(evm_address_hash))
        }
    }
}

impl Secp256k1 {
    /// Verify that a signature matches given public key.
    ///
    /// # Arguments
    ///
    /// * `public_key`: [PublicKey] - The public key to verify against.
    /// * `message`: Message - The signed data.
    ///
    /// # Returns
    ///
    /// * [Result<(), CryptographicError>] - An Ok result or an error.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::{Message, PublicKey, Secp256k1};
    ///
    /// fn foo() {
    ///     let hi = 0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c;
    ///     let lo = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    ///     let msg_hash = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    ///     let pub_hi = 0xD73A188181464CC84AE267E45041AEF6AB938F278E636AA1D02D3014C1BEF74E;
    ///     let pub_lo = 0xC44415635160ACFC87A84300EED97928C949A2D958FC0947C535F7539C59AE75;
    ///     let signature: Secp256k1 = Secp256k1::from((hi, lo));
    ///     let message: Message = Message::from(msg_hash);
    ///     let public_key: PublicKey = PublicKey::from((pub_hi, pub_lo));
    ///
    ///     // A valid result
    ///     let result = signature.verify(public_key, message);
    ///     assert(result.is_ok());
    /// }
    /// ```
    pub fn verify(self, public_key: PublicKey, message: Message) -> Result<(), CryptographicError> {
        let pub_key_result = Self::recover(self, message);

        if let Err(e) = pub_key_result {
            // propagate the error if it exists
            Err(e)
        } else if pub_key_result.unwrap() == public_key {
            Ok(())
        } else {
            Err(CryptographicError::InvalidSignature)
        }
    }

    /// Verify that an evm address matches given public key.
    ///
    /// # Arguments
    ///
    /// * `address`: [Address] - The address to verify against.
    /// * `message`: Message - The signed data.
    ///
    /// # Returns
    ///
    /// * [Result<(), CryptographicError>] - An Ok result or an error.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::{Message, Secp256k1};
    ///
    /// fn foo() {
    ///     let hi = 0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c;
    ///     let lo = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    ///     let msg_hash = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    ///     let signature: Secp256k1 = Secp256k1::from((hi, lo));
    ///     let message: Message = Message::from(msg_hash);
    ///     let address = Address::from(0xD73A188181464CC84AE267E45041AEF6AB938F278E636AA1D02D3014C1BEF74E);
    ///
    ///     // A valid result
    ///     let result = signature.verify_address(address, message);
    ///     assert(result.is_ok());
    /// }
    /// ```
    pub fn verify_address(self, address: Address, message: Message) -> Result<(), CryptographicError> {
        let address_result = Self::address(self, message);

        if let Err(e) = address_result {
            // propagate the error if it exists
            Err(e)
        } else if address_result.unwrap() == address {
            Ok(())
        } else {
            Err(CryptographicError::InvalidSignature)
        }
    }

    /// Verify that an address matches given public key.
    ///
    /// # Arguments
    ///
    /// * `evm_address`: [EvmAddress] - The evm address to verify against.
    /// * `message`: Message - The signed data.
    ///
    /// # Returns
    ///
    /// * [Result<(), CryptographicError>] - An Ok result or an error.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{crypto::{Message, Secp256k1}, vm::evm::evm_address::EvmAddress};
    ///
    /// fn foo() {
    ///     let hi = 0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c;
    ///     let lo = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    ///     let msg_hash = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;
    ///     let signature: Secp256k1 = Secp256k1::from((hi, lo));
    ///     let message: Message = Message::from(msg_hash);
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
) -> Result<(), CryptographicError> {
        let evm_address_result = Self::evm_address(self, message);

        if let Err(e) = evm_address_result {
            // propagate the error if it exists
            Err(e)
        } else if evm_address_result.unwrap() == evm_address {
            Ok(())
        } else {
            Err(CryptographicError::InvalidSignature)
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

impl From<Secp256k1> for B512 {
    fn from(signature: Secp256k1) -> Self {
        let b256_1 = asm(bits: signature.bits()) {
            bits: b256
        };
        let b256_2 = asm(bits: signature.bits()[32]) {
            bits: b256
        };
        B512::from((b256_1, b256_2))
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

impl From<Secp256k1> for (b256, b256) {
    fn from(signature: Secp256k1) -> (b256, b256) {
        let b256_1 = asm(bits: signature.bits()) {
            bits: b256
        };
        let b256_2 = asm(bits: signature.bits()[32]) {
            bits: b256
        };
        (b256_1, b256_2)
    }
}

impl core::ops::Eq for Secp256k1 {
    fn eq(self, other: Self) -> bool {
        let self_b256_1 = asm(bits: self.bits) {
            bits: b256
        };
        let self_b256_2 = asm(bits: self.bits[32]) {
            bits: b256
        };
        let other_b256_1 = asm(bits: other.bits) {
            bits: b256
        };
        let other_b256_2 = asm(bits: other.bits[32]) {
            bits: b256
        };

        self_b256_1 == other_b256_1 && self_b256_2 == other_b256_2
    }
}

impl Hash for Secp256k1 {
    fn hash(self, ref mut state: Hasher) {
        let ptr = alloc_bytes(64); // eight word capacity
        let (word_1, word_2, word_3, word_4, word_5, word_6, word_7, word_8) = asm(r1: self) {
            r1: (u64, u64, u64, u64, u64, u64, u64, u64)
        };

        asm(
            ptr: ptr,
            val_1: word_1,
            val_2: word_2,
            val_3: word_3,
            val_4: word_4,
            val_5: word_5,
            val_6: word_6,
            val_7: word_7,
            val_8: word_8,
        ) {
            sw ptr val_1 i0;
            sw ptr val_2 i1;
            sw ptr val_3 i2;
            sw ptr val_4 i3;
            sw ptr val_5 i4;
            sw ptr val_6 i5;
            sw ptr val_7 i6;
            sw ptr val_8 i7;
        };

        state.write(Bytes::from(raw_slice::from_parts::<u8>(ptr, 64)));
    }
}
