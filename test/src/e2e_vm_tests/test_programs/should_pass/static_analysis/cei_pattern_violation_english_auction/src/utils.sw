library utils;

dep data_structures/auction_asset;
dep data_structures/auction;
dep data_structures/nft_asset;
dep data_structures/token_asset;
dep errors;
dep interface;

use auction_asset::AuctionAsset;
use auction::Auction;
use errors::AccessError;
use interface::NFT;
use nft_asset::NFTAsset;
use std::{context::call_frames::contract_id, token::transfer};
use token_asset::TokenAsset;

/// Transfers assets out of the auction contract to the specified user.
///
/// # Arguments
///
/// * `asset` - The asset that is to be transfered.
/// * `to` - The user which will recieve the asset.
pub fn transfer_asset(asset: AuctionAsset, to: Identity) {
    match asset {
        AuctionAsset::NFTAsset(asset) => {
            transfer_nft(asset, Identity::ContractId(contract_id()), to)
        },
        AuctionAsset::TokenAsset(asset) => {
            transfer(asset.amount(), asset.asset_id(), to)
        },
    }
}

/// Transfers an NFT from one `Identity` to another.
///
/// # Arguments
///
/// * `asset` - The struct which contains the NFT data.
/// * `from` - The owner of the NFT.
/// * `to` - The user which the NFTs should be transfered to.
///
/// # Reverts
///
/// * The NFT transfer failed.
pub fn transfer_nft(asset: NFTAsset, from: Identity, to: Identity) {
    let nft_abi = abi(NFT, asset.asset_id().value);

    nft_abi.transfer_from(from, to, asset.token_id());

    let owner = nft_abi.owner_of(asset.token_id());
    require(owner == to, AccessError::NFTTransferNotApproved);
}
