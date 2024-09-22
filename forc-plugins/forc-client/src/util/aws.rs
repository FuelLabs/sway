use async_trait::async_trait;
use aws_config::{default_provider::credentials::DefaultCredentialsChain, Region, SdkConfig};
use aws_sdk_kms::config::Credentials;
use aws_sdk_kms::operation::get_public_key::GetPublicKeyOutput;
use aws_sdk_kms::primitives::Blob;
use aws_sdk_kms::types::{MessageType, SigningAlgorithmSpec};
use aws_sdk_kms::{config::BehaviorVersion, Client};
use fuel_crypto::coins_bip32::prelude::k256::pkcs8::spki;
use fuel_crypto::{Message, PublicKey};
use fuels::types::bech32::{Bech32Address, FUEL_BECH32_HRP};
use fuels_core::traits::Signer;

#[derive(Debug, Clone)]
pub struct AwsConfig {
    sdk_config: SdkConfig,
}

impl AwsConfig {
    pub async fn from_env() -> Self {
        let loader = aws_config::defaults(BehaviorVersion::latest())
            .credentials_provider(DefaultCredentialsChain::builder().build().await);

        let loader = match std::env::var("E2E_TEST_AWS_ENDPOINT") {
            Ok(url) => loader.endpoint_url(url),
            _ => loader,
        };

        Self {
            sdk_config: loader.load().await,
        }
    }

    pub async fn for_testing(url: String) -> Self {
        let sdk_config = aws_config::defaults(BehaviorVersion::latest())
            .credentials_provider(Credentials::new(
                "test",
                "test",
                None,
                None,
                "Static Credentials",
            ))
            .endpoint_url(url)
            .region(Region::new("us-east-1")) // placeholder region for test
            .load()
            .await;

        Self { sdk_config }
    }

    pub fn url(&self) -> Option<&str> {
        self.sdk_config.endpoint_url()
    }

    pub fn region(&self) -> Option<&Region> {
        self.sdk_config.region()
    }
}

#[derive(Clone)]
pub struct AwsClient {
    client: Client,
}

impl AwsClient {
    pub async fn new(config: AwsConfig) -> Self {
        let config = config.sdk_config;
        let client = Client::new(&config);

        Self { client }
    }

    pub fn inner(&self) -> &Client {
        &self.client
    }
}

#[derive(Clone)]
pub struct AwsSigner {
    kms: Client,
    key_id: String,
    public_key_bytes: Vec<u8>,
    bech: Bech32Address,
}

async fn request_get_pubkey(
    kms: &Client,
    key_id: String,
) -> Result<GetPublicKeyOutput, anyhow::Error> {
    kms.get_public_key()
        .key_id(key_id)
        .send()
        .await
        .map_err(Into::into)
}

/// Decode an AWS KMS Pubkey response.
fn decode_pubkey(resp: &GetPublicKeyOutput) -> Result<&[u8], anyhow::Error> {
    let raw = resp
        .public_key
        .as_ref()
        .ok_or(anyhow::anyhow!("public key not found"))?;
    let spki = spki::SubjectPublicKeyInfoRef::try_from(raw.as_ref())?;
    let bytes = spki.subject_public_key.raw_bytes();
    Ok(bytes)
}

async fn sign_with_kms(
    client: &aws_sdk_kms::Client,
    key_id: &str,
    public_key_bytes: &[u8],
    message: Message,
) -> anyhow::Result<fuel_crypto::Signature> {
    use k256::{
        ecdsa::{RecoveryId, VerifyingKey},
        pkcs8::DecodePublicKey,
    };

    let reply = client
        .sign()
        .key_id(key_id)
        .signing_algorithm(SigningAlgorithmSpec::EcdsaSha256)
        .message_type(MessageType::Digest)
        .message(Blob::new(*message))
        .send()
        .await
        .inspect_err(|err| tracing::error!("Failed to sign with AWS KMS: {err:?}"))?;
    let signature_der = reply
        .signature
        .ok_or_else(|| anyhow::anyhow!("no signature returned from AWS KMS"))?
        .into_inner();
    // https://stackoverflow.com/a/71475108
    let sig = k256::ecdsa::Signature::from_der(&signature_der)
        .map_err(|_| anyhow::anyhow!("invalid DER signature from AWS KMS"))?;
    let sig = sig.normalize_s().unwrap_or(sig);

    // This is a hack to get the recovery id. The signature should be normalized
    // before computing the recovery id, but aws kms doesn't support this, and
    // instead always computes the recovery id from non-normalized signature.
    // So instead the recovery id is determined by checking which variant matches
    // the original public key.

    let recid1 = RecoveryId::new(false, false);
    let recid2 = RecoveryId::new(true, false);

    let rec1 = VerifyingKey::recover_from_prehash(&*message, &sig, recid1);
    let rec2 = VerifyingKey::recover_from_prehash(&*message, &sig, recid2);

    let correct_public_key = k256::PublicKey::from_public_key_der(public_key_bytes)
        .map_err(|_| anyhow::anyhow!("invalid DER public key from AWS KMS"))?
        .into();

    let recovery_id = if rec1.map(|r| r == correct_public_key).unwrap_or(false) {
        recid1
    } else if rec2.map(|r| r == correct_public_key).unwrap_or(false) {
        recid2
    } else {
        anyhow::bail!("Invalid signature generated (reduced-x form coordinate)");
    };

    // Insert the recovery id into the signature
    debug_assert!(
        !recovery_id.is_x_reduced(),
        "reduced-x form coordinates are caught by the if-else chain above"
    );
    let v = recovery_id.is_y_odd() as u8;
    let mut signature = <[u8; 64]>::from(sig.to_bytes());
    signature[32] = (v << 7) | (signature[32] & 0x7f);
    Ok(fuel_crypto::Signature::from_bytes(signature))
}

impl AwsSigner {
    pub async fn new(kms: Client, key_id: String) -> Result<Self, anyhow::Error> {
        let resp = request_get_pubkey(&kms, key_id.clone()).await?;
        let public_key_bytes = decode_pubkey(&resp)?.to_vec();
        let public_key = PublicKey::try_from(public_key_bytes.as_slice()).unwrap();
        let hashed = public_key.hash();
        let bech = Bech32Address::new(FUEL_BECH32_HRP, hashed);
        Ok(Self {
            kms,
            key_id,
            public_key_bytes,
            bech,
        })
    }

    /// Sign a digest with the key associated with a key ID.
    pub async fn sign_message_with_key(
        &self,
        key_id: String,
        message: Message,
    ) -> Result<fuel_crypto::Signature, anyhow::Error> {
        sign_with_kms(&self.kms, &key_id, &self.public_key_bytes, message).await
    }

    /// Sign a digest with this signer's key
    pub async fn sign_message(
        &self,
        message: Message,
    ) -> Result<fuel_crypto::Signature, anyhow::Error> {
        self.sign_message_with_key(self.key_id.clone(), message)
            .await
    }
}

#[async_trait]
impl Signer for AwsSigner {
    async fn sign(
        &self,
        message: Message,
    ) -> Result<fuel_crypto::Signature, fuels_core::types::errors::Error> {
        let sig = self.sign_message(message).await.map_err(|_| {
            fuels_core::types::errors::Error::Other("aws signer failed".to_string())
        })?;
        Ok(sig)
    }

    fn address(&self) -> &Bech32Address {
        &self.bech
    }
}
