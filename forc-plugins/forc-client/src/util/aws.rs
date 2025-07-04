use async_trait::async_trait;
use aws_config::{default_provider::credentials::DefaultCredentialsChain, Region, SdkConfig};
use aws_sdk_kms::config::Credentials;
use aws_sdk_kms::operation::get_public_key::GetPublicKeyOutput;
use aws_sdk_kms::primitives::Blob;
use aws_sdk_kms::types::{MessageType, SigningAlgorithmSpec};
use aws_sdk_kms::{config::BehaviorVersion, Client};
use fuel_crypto::Message;
use fuels::prelude::*;
use fuels::types::coin_type_id::CoinTypeId;
use fuels::types::input::Input;
use fuels_accounts::provider::Provider;
use fuels_accounts::{Account, ViewOnlyAccount};
use fuels_core::traits::Signer;

/// AWS configuration for the `AwsSigner` to be created.
/// De-facto way of creating the configuration is to load it from env.
#[derive(Debug, Clone)]
pub struct AwsConfig {
    sdk_config: SdkConfig,
}

impl AwsConfig {
    /// Load configuration from environment variables.
    /// For more details see: https://docs.rs/aws-config/latest/aws_config/
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

/// A configured `AwsClient` which allows using the AWS KMS SDK.
#[derive(Clone, Debug)]
pub struct AwsClient {
    client: Client,
}

impl AwsClient {
    pub fn new(config: AwsConfig) -> Self {
        let config = config.sdk_config;
        let client = Client::new(&config);

        Self { client }
    }

    pub fn inner(&self) -> &Client {
        &self.client
    }
}

/// A signer which is capable of signing `fuel_crypto::Message`s using AWS KMS.
/// This is both a `Signer` and `Account`, which means it is directly usable
/// with most of the fuels-* calls, without any additional operations on the
/// representation.
#[derive(Clone, Debug)]
pub struct AwsSigner {
    kms: AwsClient,
    key_id: String,
    address: Address,
    public_key_bytes: Vec<u8>,
    provider: Provider,
}

async fn request_get_pubkey(
    kms: &Client,
    key_id: String,
) -> std::result::Result<GetPublicKeyOutput, anyhow::Error> {
    kms.get_public_key()
        .key_id(key_id)
        .send()
        .await
        .map_err(Into::into)
}

/// Decode an AWS KMS Pubkey response.
fn decode_pubkey(resp: &GetPublicKeyOutput) -> std::result::Result<Vec<u8>, anyhow::Error> {
    let raw = resp
        .public_key
        .as_ref()
        .ok_or(anyhow::anyhow!("public key not found"))?;
    Ok(raw.clone().into_inner())
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
    pub async fn new(
        kms: AwsClient,
        key_id: String,
        provider: Provider,
    ) -> std::result::Result<Self, anyhow::Error> {
        use k256::pkcs8::DecodePublicKey;

        let resp = request_get_pubkey(kms.inner(), key_id.clone()).await?;
        let public_key_bytes = decode_pubkey(&resp)?;
        let k256_public_key = k256::PublicKey::from_public_key_der(&public_key_bytes)?;

        let public_key = fuel_crypto::PublicKey::from(k256_public_key);
        let hashed = public_key.hash();
        let address = Address::from(*hashed);
        Ok(Self {
            kms,
            key_id,
            address,
            public_key_bytes,
            provider,
        })
    }

    /// Sign a digest with the key associated with a key ID.
    pub async fn sign_message_with_key(
        &self,
        key_id: String,
        message: Message,
    ) -> std::result::Result<fuel_crypto::Signature, anyhow::Error> {
        sign_with_kms(self.kms.inner(), &key_id, &self.public_key_bytes, message).await
    }

    /// Sign a digest with this signer's key.
    pub async fn sign_message(
        &self,
        message: Message,
    ) -> std::result::Result<fuel_crypto::Signature, anyhow::Error> {
        self.sign_message_with_key(self.key_id.clone(), message)
            .await
    }

    pub fn provider(&self) -> &Provider {
        &self.provider
    }
}

#[async_trait]
impl Signer for AwsSigner {
    async fn sign(&self, message: Message) -> Result<fuel_crypto::Signature> {
        let sig = self.sign_message(message).await.map_err(|_| {
            fuels_core::types::errors::Error::Other("aws signer failed".to_string())
        })?;
        Ok(sig)
    }

    fn address(&self) -> Address {
        self.address
    }
}

#[async_trait]
impl ViewOnlyAccount for AwsSigner {
    fn address(&self) -> Address {
        self.address
    }

    fn try_provider(&self) -> Result<&Provider> {
        Ok(&self.provider)
    }

    async fn get_asset_inputs_for_amount(
        &self,
        asset_id: AssetId,
        amount: u128,
        excluded_coins: Option<Vec<CoinTypeId>>,
    ) -> Result<Vec<Input>> {
        Ok(self
            .get_spendable_resources(asset_id, amount, excluded_coins)
            .await?
            .into_iter()
            .map(Input::resource_signed)
            .collect::<Vec<Input>>())
    }
}

#[async_trait]
impl Account for AwsSigner {}
