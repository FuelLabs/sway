contract;

dep data_structures/state;
dep errors;
dep events;
dep interface;
dep utils;

// TODO: Move these into alphabetical order once https://github.com/FuelLabs/sway/issues/409 is resolved
dep data_structures/auction_asset;
dep data_structures/auction;
dep data_structures/token_asset;
dep data_structures/nft_asset;

use auction_asset::AuctionAsset;
use auction::Auction;
use errors::{AccessError, InitError, InputError, UserError};
use events::{BidEvent, CancelAuctionEvent, CreateAuctionEvent, WithdrawEvent};
use interface::{EnglishAuction, NFT};
use nft_asset::NFTAsset;
use state::State;
use std::{
    block::height,
    chain::auth::msg_sender,
    context::{
        call_frames::{
            contract_id,
            msg_asset_id,
        },
        msg_amount,
    },
    logging::log,
    storage::StorageMap,
};
use token_asset::TokenAsset;
use utils::{transfer_asset, transfer_nft};

storage {
    /// Stores the auction information based on auction ID.
    /// Map(auction id => auction)
    auctions: StorageMap<u64, Option<Auction>> = StorageMap {}, 
    
    // TODO: Move deposits into the Auction struct when StorageMaps are
    //       supported inside structs
    /// Stores the deposits made based on the user and auction ID.
    /// Map((user, auction id) => deposit)
    deposits: StorageMap<(Identity, u64), Option<AuctionAsset>> = StorageMap {},
    /// The total number of auctions that have ever been created.
    total_auctions: u64 = 0,
}

impl EnglishAuction for Contract {
    #[storage(read)]
    fn auction_info(auction_id: u64) -> Option<Auction> {
        storage.auctions.get(auction_id)
    }

    #[storage(read, write)]
    fn bid(auction_id: u64, bid_asset: AuctionAsset) {
        let auction = storage.auctions.get(auction_id);
        require(auction.is_some(), InputError::AuctionDoesNotExist);

        let mut auction = auction.unwrap();
        let sender = msg_sender().unwrap();
        require(sender != auction.seller, UserError::BidderIsSeller);
        require(auction.state == State::Open && height() <= auction.end_block, AccessError::AuctionIsNotOpen);
        require(bid_asset == auction.bid_asset, InputError::IncorrectAssetProvided);

        // Combine the user's previous deposits and the current bid for the
        // total deposits to the auction the user has made
        let sender_deposit = storage.deposits.get((sender, auction_id));
        let total_bid = match sender_deposit {
            Option::Some(AuctionAsset) => {
                bid_asset + sender_deposit.unwrap()
            },
            Option::None(AuctionAsset) => {
                bid_asset
            }
        };

        match total_bid {
            AuctionAsset::NFTAsset(nft_asset) => {
                transfer_nft(nft_asset, sender, Identity::ContractId(contract_id()));
                // TODO: Remove this once StorageVec is supported in structs
                auction.state = State::Closed;
            },
            AuctionAsset::TokenAsset(token_asset) => {
                require(bid_asset.amount() == msg_amount(), InputError::IncorrectAmountProvided);
                require(bid_asset.asset_id() == msg_asset_id(), InputError::IncorrectAssetProvided);
                // Ensure this bid is greater than initial bid and the total deposits are greater 
                // than the current winnning bid
                // TODO: Move this outside the match statement once StorageVec is supported in structs
                require(token_asset.amount() >= auction.initial_price, InputError::InitialPriceNotMet);
                require(token_asset.amount() > auction.bid_asset.amount(), InputError::IncorrectAmountProvided);
            }
        }

        // Check if reserve has been met if there is one set
        if auction.reserve_price.is_some() {
            // The bid cannot be greater than the reserve price
            require(auction.reserve_price.unwrap() >= total_bid.amount(), InputError::IncorrectAmountProvided);

            if auction.reserve_price.unwrap() == total_bid.amount() {
                auction.state = State::Closed;
            }
        }

        // Update the auction's information and store the new state
        auction.highest_bidder = Option::Some(sender);
        auction.bid_asset = total_bid;
        storage.deposits.insert((sender, auction_id), Option::Some(auction.bid_asset));
        storage.auctions.insert(auction_id, Option::Some(auction));

        log(BidEvent {
            amount: auction.bid_asset.amount(),
            auction_id: auction_id,
            user: sender,
        });
    }

    #[storage(read, write)]
    fn cancel(auction_id: u64) {
        // Make sure this auction exists
        let auction = storage.auctions.get(auction_id);
        require(auction.is_some(), InputError::AuctionDoesNotExist);

        let mut auction = auction.unwrap();
        require(auction.state == State::Open && height() <= auction.end_block, AccessError::AuctionIsNotOpen);
        require(msg_sender().unwrap() == auction.seller, AccessError::SenderIsNotSeller);

        // Update and store the auction's information
        auction.highest_bidder = Option::None();
        auction.state = State::Closed;
        storage.auctions.insert(auction_id, Option::Some(auction));

        log(CancelAuctionEvent { auction_id });
    }

    #[storage(read, write)]
    fn create(
        bid_asset: AuctionAsset,
        duration: u64,
        initial_price: u64,
        reserve_price: Option<u64>,
        seller: Identity,
        sell_asset: AuctionAsset,
    ) -> u64 {
        // Either there is no reserve price or the reserve must be greater than the initial price
        require(reserve_price.is_none() || (reserve_price.is_some() && reserve_price.unwrap() >= initial_price), InitError::ReserveLessThanInitialPrice);
        require(duration != 0, InitError::AuctionDurationNotProvided);

        // TODO: This will be combined once StorageVec is supported in structs
        match bid_asset {
            AuctionAsset::TokenAsset(asset) => {
                require(asset.amount() == 0, InitError::BidAssetAmountNotZero);
            },
            AuctionAsset::NFTAsset(asset) => {
                require(asset.token_id() == 0, InitError::BidAssetAmountNotZero);
            }
        }

        // Ensure that the `sell_asset` struct and what was sent in the transaction match
        match sell_asset {
            AuctionAsset::TokenAsset(asset) => {
                // Selling tokens
                // TODO: Move this outside the match statement when StorageVec in structs is supported
                require(initial_price != 0, InitError::InitialPriceCannotBeZero);
                require(msg_amount() == asset.amount(), InputError::IncorrectAmountProvided);
                require(msg_asset_id() == asset.asset_id(), InputError::IncorrectAssetProvided);
            },
            AuctionAsset::NFTAsset(asset) => {
                // Selling NFTs
                let sender = msg_sender().unwrap();
                // TODO: Remove this when StorageVec in structs is supported
                require(initial_price == 1, InitError::CannotAcceptMoreThanOneNFT);
                transfer_nft(asset, sender, Identity::ContractId(contract_id()));
            }
        }

        // Setup auction
        let auction = ~Auction::new(bid_asset, height() + duration, initial_price, reserve_price, sell_asset, seller);

        // Store the auction information
        let total_auctions = storage.total_auctions;
        storage.deposits.insert((seller, total_auctions), Option::Some(sell_asset));
        storage.auctions.insert(total_auctions, Option::Some(auction));

        log(CreateAuctionEvent {
            auction_id: total_auctions,
            bid_asset,
            sell_asset,
        });

        storage.total_auctions += 1;
        total_auctions
    }

    #[storage(read)]
    fn deposit_balance(auction_id: u64, identity: Identity) -> Option<AuctionAsset> {
        storage.deposits.get((identity, auction_id))
    }

    #[storage(read, write)]
    fn withdraw(auction_id: u64) {
        // Make sure this auction exists
        let auction = storage.auctions.get(auction_id);
        require(auction.is_some(), InputError::AuctionDoesNotExist);

        // Cannot withdraw if the auction is still on going
        let mut auction = auction.unwrap();
        require(auction.state == State::Closed || height() >= auction.end_block, AccessError::AuctionIsNotClosed);
        if (height() >= auction.end_block
            && auction.state == State::Open)
        {
            auction.state = State::Closed;
            storage.auctions.insert(auction_id, Option::Some(auction));
        }

        let sender = msg_sender().unwrap();
        let bidder = auction.highest_bidder;
        let sender_deposit = storage.deposits.get((sender, auction_id));

        // Make sure the sender still has something to withdraw
        require(sender_deposit.is_some(), UserError::UserHasAlreadyWithdrawn);
        storage.deposits.insert((sender, auction_id), Option::None::<AuctionAsset>());
        let mut withdrawn_asset = sender_deposit.unwrap();

        // Withdraw owed assets
        if ((bidder.is_some()
            && sender == bidder.unwrap())
            || (bidder.is_none()
            && sender == auction.seller))
        {
            transfer_asset(auction.sell_asset, sender);
            withdrawn_asset = auction.sell_asset;
        } else if (sender == auction.seller) {
            transfer_asset(auction.bid_asset, sender);
            withdrawn_asset = auction.bid_asset;
        } else {
            transfer_asset(sender_deposit.unwrap(), sender);
        };

        log(WithdrawEvent {
            asset: withdrawn_asset,
            auction_id,
            user: sender,
        });
    }

    #[storage(read)]
    fn total_auctions() -> u64 {
        storage.total_auctions
    }
}
