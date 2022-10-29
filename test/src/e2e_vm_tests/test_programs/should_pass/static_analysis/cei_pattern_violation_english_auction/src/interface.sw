library interface;

dep data_structures/auction_asset;
dep data_structures/auction;

use auction_asset::AuctionAsset;
use auction::Auction;

abi EnglishAuction {
    /// Returns the auction struct for the corresponding auction id.
    ///
    /// # Arguments
    ///
    /// * `auction_id` - The id number of the auction.
    #[storage(read)]
    fn auction_info(auction_id: u64) -> Option<Auction>;

    /// Places a bid on the specified auction.
    ///
    /// # Arguments
    ///
    /// * `auction_id` - The id number of the auction.
    /// * `bid_asset` - An asset that is either a `TokenAsset` struct or a `NFTAsset` struct.
    ///
    /// # Reverts
    ///
    /// * When the `auction_id` does not map to an existing auction.
    /// * When the auction has closed.
    /// * When the bidding period for the auction has passed.
    /// * When the asset provided does not match the asset accepted for the auction.
    /// * When the bidder is the auction's seller.
    /// * When transfering of the NFT asset to the auction contract failed.
    /// * When the native asset amount sent and the `bid_asset` enum do not match.
    /// * When the native asset type sent and the `bid_asset` enum do not match.
    /// * When the bid amount is less than the initial price.
    /// * When the bidder's total deposits are not greater than the current bid.
    /// * When the bidder's total deposits are greater than the reserve price.
    #[storage(read, write)]
    fn bid(auction_id: u64, bid_asset: AuctionAsset);

    /// Cancels the specified auction.
    ///
    /// # Arguments
    ///
    /// * `auction_id` - The `u64` id number of the auction.
    ///
    /// # Reverts
    ///
    /// * When the `auction_id` does not map to an existing auction.
    /// * When the auction is no longer open.
    /// * When the sender is not the seller of the auction.
    #[storage(read, write)]
    fn cancel(auction_id: u64);

    /// Starts an auction with a seller, selling asset, accepted bid asset, initial price, a
    /// possible reserve price, and duration of the auction.
    ///
    /// This function will return the newly created auction's ID number which is used to
    /// interact with the auction.
    ///
    /// # Arguments
    ///
    /// `bid_asset` - The asset the seller is willing to accept in return for the selling asset.
    /// `duration` - The length of time the auction should be open.
    /// `initial_price` - The starting price at which the auction should start.
    /// `reserve_price` - The price at which a buyer may purchase the `sell_asset` outright.
    /// `seller` - The seller for this auction.
    /// `sell_asset` - The enum that contains information about what is being auctioned off.
    ///
    /// # Reverts
    ///
    /// * When the `reserve_price` is less than `initial_price` and a reserve is set.
    /// * When the `duration` of the auction is set to zero.
    /// * When the `bid_asset` amount is not zero.
    /// * When the `initial_price` for tokens is set to zero.
    /// * When the native asset amount sent and the `sell_asset` enum do not match.
    /// * When the native asset type sent and the `sell_asset` enum do not match.
    /// * When the `initial_price` for NFTs is not one.
    /// * When transfering of the NFT asset to the contract failed.
    #[storage(read, write)]
    fn create(bid_asset: AuctionAsset, duration: u64, inital_price: u64, reserve_price: Option<u64>, seller: Identity, sell_asset: AuctionAsset) -> u64;

    /// Returns the balance of the user's deposits for the specified auction.
    ///
    /// # Arguments
    ///
    /// * `identity` - The user which has deposited assets.
    /// * `auction_id` - The id number of the auction.
    #[storage(read)]
    fn deposit_balance(auction_id: u64, identity: Identity) -> Option<AuctionAsset>;

    /// Allows users to withdraw their owed assets if the auction's bid period has ended, the
    /// reserve has been met, or the auction has been canceled.
    ///
    /// 1. If the sender is the winning bidder, they will withdraw the selling asset.
    /// 2. If the sender's bids failed to win the auction, their total deposits will be withdrawn.
    /// 3. If the sender is the seller and no bids have been made or the auction has been canceled,
    ///    they will withdraw the selling asset.
    /// 4. If the sender is the seller and a bid has been made, they will withdraw the winning
    ///    bidder's total deposits.
    ///
    /// # Arguments
    ///
    /// * `auction_id` - The id number of the auction.
    ///
    /// # Reverts
    ///
    /// * When the `auction_id` provided does not map to an existing auction.
    /// * When the bidding period of the auction has not ended.
    /// * When the auction's `state` is still in the open bidding state.
    /// * When the sender has already withdrawn their deposit.
    #[storage(read, write)]
    fn withdraw(auction_id: u64);

    /// Returns the total auctions which have been started using this auction contract.
    #[storage(read)]
    fn total_auctions() -> u64;
}

abi NFT {
    fn owner_of(token_id: u64) -> Identity;
    fn transfer_from(from: Identity, to: Identity, token_id: u64);
}
