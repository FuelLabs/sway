library auction;

dep auction_asset;
dep state;

use auction_asset::AuctionAsset;
use state::State;

pub struct Auction {
    /// The asset which will be accepted in return for the selling asset.
    bid_asset: AuctionAsset,
    /// The block at which the auction's bidding period should end.
    end_block: u64,
    /// The current highest bidder of the auction.
    highest_bidder: Option<Identity>,
    /// The starting price for the auction.
    initial_price: u64,
    /// The price at which the selling asset may be bought outright.
    reserve_price: Option<u64>,
    /// The asset that is being auctioned off.
    sell_asset: AuctionAsset,
    /// The seller of the auction.
    seller: Identity,
    /// The state of the auction describing if it is open or closed.
    state: State,
}

impl Auction {
    fn new(
        bid_asset: AuctionAsset,
        end_block: u64,
        initial_price: u64,
        reserve_price: Option<u64>,
        sell_asset: AuctionAsset,
        seller: Identity,
    ) -> Self {
        Auction {
            bid_asset,
            end_block,
            highest_bidder: Option::None(),
            initial_price,
            reserve_price,
            sell_asset,
            seller,
            state: State::Open,
        }
    }
}
