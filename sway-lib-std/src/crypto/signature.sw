library;

use ::address::Address;
use ::crypto::{
    ed25519::Ed25519,
    message::Message,
    public_key::PublicKey,
    secp256k1::Secp256k1,
    secp256r1::Secp256r1,
    signature_error::SignatureError,
};
use ::option::Option::{self, *};
use ::result::Result::{self, *};
use ::vm::evm::evm_address::EvmAddress;
use ::codec::*;
use ::ops::*;

/// An ECDSA signature.
pub enum Signature {
    Secp256k1: Secp256k1,
    Secp256r1: Secp256r1,
    Ed25519: Ed25519,
}

impl Signature {
    /// Recover the public key derived from the private key used to sign a message.
    /// Returns a `Result` to let the caller choose an error handling strategy.
    ///
    /// # Additional Information
    ///
    /// Not applicable for Ed25519 signatures.
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
    /// use std::crypto::{Message, PublicKey, Secp256r1, Signature};
    ///
    /// fn foo() {
    ///     let signature: Signature = Signature::Secp256r1(Secp256r1::from((
    ///         0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c,
    ///         0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d
    ///     )));
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
        match self {
            Self::Secp256k1(sig) => {
                sig.recover(message)
            },
            Self::Secp256r1(sig) => {
                sig.recover(message)
            },
            Self::Ed25519(_) => {
                Err(SignatureError::InvalidOperation)
            },
        }
    }

    /// Recover the address derived from the private key used to sign a message.
    /// Returns a `Result` to let the caller choose an error handling strategy.
    ///
    /// # Additional Information
    ///
    /// Not applicable for Ed25519 signatures.
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
    /// use std::crypto::{Message, Secp256r1, Signature};
    ///
    /// fn foo() {
    ///     let address = Address::from(0x7AAE2D980BE4C3275C72CE5B527FA23FFB97B766966559DD062E2B78FD9D3766);
    ///     let signature: Signature = Signature::Secp256r1(Secp256r1::from((
    ///         0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c,
    ///         0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d
    ///     )));
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
        match self {
            Self::Secp256k1(sig) => {
                sig.address(message)
            },
            Self::Secp256r1(sig) => {
                sig.address(message)
            },
            Self::Ed25519(_) => {
                Err(SignatureError::InvalidOperation)
            },
        }
    }

    /// Recover the EVM address derived from the private key used to sign a message.
    /// Returns a `Result` to let the caller choose an error handling strategy.
    ///
    /// # Additional Information
    ///
    /// Not applicable for Ed25519 signatures.
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
    /// use std::{vm::evm::evm_address::EvmAddress, crypto::{Signature, Secp256k1, Message}};
    ///
    /// fn foo() {
    ///     let evm_address = EvmAddress::from(0x7AAE2D980BE4C3275C72CE5B527FA23FFB97B766966559DD062E2B78FD9D3766);
    ///     let signature: Signature = Signature::Secp256k1(Secp256k1::from((
    ///         0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c,
    ///         0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d
    ///     )));
    ///     let message: Message = Message::from(0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323);
    ///     // A recovered evm address.
    ///     let result_address = signature.evm_address(message).unwrap();
    ///     assert(result_address.is_ok());
    ///     assert(result_address.unwrap() == evm_address);
    /// }
    /// ```
    pub fn evm_address(self, message: Message) -> Result<EvmAddress, SignatureError> {
        match self {
            Self::Secp256k1(sig) => {
                sig.evm_address(message)
            },
            Self::Secp256r1(sig) => {
                sig.evm_address(message)
            },
            Self::Ed25519(_) => {
                Err(SignatureError::InvalidOperation)
            },
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
    /// use std::crypto::{Message, PublicKey, Secp256r1, Signature};
    ///
    /// fn foo() {
    ///     let signature: Signature = Signature::Secp256r1(Secp256r1::from((
    ///         0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c,
    ///         0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d
    ///     )));
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
        match self {
            Self::Secp256k1(sig) => {
                sig.verify(public_key, message)
            },
            Self::Secp256r1(sig) => {
                sig.verify(public_key, message)
            },
            Self::Ed25519(sig) => {
                sig.verify(public_key, message)
            },
        }
    }

    /// Verify that a signature matches given address.
    ///
    /// # Additional Information
    ///
    /// Not applicable for Ed25519 signatures.
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
    /// use std::crypto::{Message, Secp256r1, Signature};
    ///
    /// fn foo() {
    ///     let signature: Signature = Signature::Secp256r1(Secp256r1::from((
    ///         0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c,
    ///         0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d
    ///     )));
    ///     let message: Message = Message::from(0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323);
    ///     let address = Address::from(0xD73A188181464CC84AE267E45041AEF6AB938F278E636AA1D02D3014C1BEF74E);
    ///
    ///     // A valid result
    ///     let result = signature.verify_address(address, message);
    ///     assert(result.is_ok());
    /// }
    /// ```
    pub fn verify_address(self, address: Address, message: Message) -> Result<(), SignatureError> {
        match self {
            Self::Secp256k1(sig) => {
                sig.verify_address(address, message)
            },
            Self::Secp256r1(sig) => {
                sig.verify_address(address, message)
            },
            Self::Ed25519(_) => {
                Err(SignatureError::InvalidOperation)
            },
        }
    }

    /// Verify that an signature matches given evm address.
    ///
    /// # Additional Information
    ///
    /// Not applicable for Ed25519 signatures.
    ///
    /// # Arguments
    ///
    /// * `evm_address`: [EvmAddress] - The evm address to verify against.
    /// * `message`: Message - The signed data.
    ///
    /// # Returns
    ///
    /// * [Result<(), SignatureError>] - An Ok result or an error.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{crypto::{Message, Secp256r1, Signature}, vm::evm::evm_address::EvmAddress};
    ///
    /// fn foo() {
    ///     let signature: Signature = Signature::Secp256r1(Secp256r1::from((
    ///         0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c,
    ///         0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d
    ///     )));
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
        match self {
            Self::Secp256k1(sig) => {
                sig.verify_evm_address(evm_address, message)
            },
            Self::Secp256r1(sig) => {
                sig.verify_evm_address(evm_address, message)
            },
            Self::Ed25519(_) => {
                Err(SignatureError::InvalidOperation)
            },
        }
    }

    /// Returns the `Secp256k1` of the `Signature`.
    ///
    /// # Returns
    ///
    /// * [Option<Secp256k1>] - `Some(Secp256k1)` if the underlying type is an `Secp256k1`, otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::{Signature, Secp256k1};
    ///
    /// fn foo() {
    ///     let signature = Signature::Secp256k1(Secp256k1::new());
    ///     let secp256k1 = signature.as_secp256k1();
    ///     assert(secp256k1 == Secp256k1::new());
    /// }
    /// ```
    pub fn as_secp256k1(self) -> Option<Secp256k1> {
        match self {
            Self::Secp256k1(sig) => Some(sig),
            Self::Secp256r1(_) => None,
            Self::Ed25519(_) => None,
        }
    }

    /// Returns the `Secp256r1` of the `Signature`.
    ///
    /// # Returns
    ///
    /// * [Option<Secp256r1>] - `Some(Secp256r1)` if the underlying type is an `Secp256r1`, otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::{Signature, Secp256r1};
    ///
    /// fn foo() {
    ///     let signature = Signature::Secp256r1(Secp256r1::new());
    ///     let secp256r1 = signature.as_secp256k1();
    ///     assert(secp256r1 == Secp256r1::new());
    /// }
    /// ```
    pub fn as_secp256r1(self) -> Option<Secp256r1> {
        match self {
            Self::Secp256r1(sig) => Some(sig),
            Self::Secp256k1(_) => None,
            Self::Ed25519(_) => None,
        }
    }

    /// Returns the `Ed25519` of the `Signature`.
    ///
    /// # Returns
    ///
    /// * [Option<Ed25519>] - `Some(Ed25519)` if the underlying type is an `Ed25519`, otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::{Signature, Ed25519};
    ///
    /// fn foo() {
    ///     let signature = Signature::Ed25519(Ed25519::new());
    ///     let ed25519 = signature.as_secp256k1();
    ///     assert(ed25519 == Ed25519::new());
    /// }
    /// ```
    pub fn as_ed25519(self) -> Option<Ed25519> {
        match self {
            Self::Ed25519(sig) => Some(sig),
            Self::Secp256k1(_) => None,
            Self::Secp256r1(_) => None,
        }
    }

    /// Returns whether the `Signature` represents an `Secp256k1`.
    ///
    /// # Returns
    ///
    /// * [bool] - Indicates whether the `Signature` holds an `Secp256k1`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::{Signature, Secp256k1};
    ///
    /// fn foo() {
    ///     let signature = Signature::Secp256k1(Secp256k1::new());
    ///     assert(signature.is_secp256k1());
    /// }
    /// ```
    pub fn is_secp256k1(self) -> bool {
        match self {
            Self::Secp256k1(_) => true,
            Self::Secp256r1(_) => false,
            Self::Ed25519(_) => false,
        }
    }

    /// Returns whether the `Signature` represents an `Secp256r1`.
    ///
    /// # Returns
    ///
    /// * [bool] - Indicates whether the `Signature` holds an `Secp256r1`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::{Signature, Secp256r1};
    ///
    /// fn foo() {
    ///     let signature = Signature::Secp256r1(Secp256r1::new());
    ///     assert(signature.is_secp256r1());
    /// }
    /// ```
    pub fn is_secp256r1(self) -> bool {
        match self {
            Self::Secp256r1(_) => true,
            Self::Secp256k1(_) => false,
            Self::Ed25519(_) => false,
        }
    }

    /// Returns whether the `Signature` represents an `Ed25519`.
    ///
    /// # Returns
    ///
    /// * [bool] - Indicates whether the `Signature` holds an `Ed25519`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::{Signature, Ed25519};
    ///
    /// fn foo() {
    ///     let signature = Signature::Ed25519(Ed25519::new());
    ///     assert(signature.is_ed25519());
    /// }
    /// ```
    pub fn is_ed25519(self) -> bool {
        match self {
            Self::Ed25519(_) => true,
            Self::Secp256k1(_) => false,
            Self::Secp256r1(_) => false,
        }
    }

    /// Returns the underlying raw `[u8; 64]` data of the Signature.
    ///
    /// # Returns
    ///
    /// * [[u8; 64]] - The raw data of the signature.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::crypto::{Signature, Ed25519};
    ///
    /// fn foo() -> {
    ///     let my_signature = Signature::Ed25519(Ed25519::new());
    ///     assert(my_signature.bits()[0] == 0u8);
    /// }
    /// ```
    pub fn bits(self) -> [u8; 64] {
        match self {
            Self::Secp256k1(sig) => sig.bits(),
            Self::Secp256r1(sig) => sig.bits(),
            Self::Ed25519(sig) => sig.bits(),
        }
    }
}

impl PartialEq for Signature {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (Self::Secp256k1(sig_1), Self::Secp256k1(sig_2)) => {
                sig_1 == sig_2
            },
            (Self::Secp256r1(sig_1), Self::Secp256r1(sig_2)) => {
                sig_1 == sig_2
            },
            (Self::Ed25519(sig_1), Self::Ed25519(sig_2)) => {
                sig_1 == sig_2
            },
            _ => false,
        }
    }
}
impl Eq for Signature {}
