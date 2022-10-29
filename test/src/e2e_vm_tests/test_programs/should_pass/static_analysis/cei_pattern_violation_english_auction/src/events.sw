library events;

dep data_structures/auction_asset;
dep data_structures/auction;

use auction_asset::AuctionAsset;
use auction::Auction;

pub struct CancelAuctionEvent {
    auction_id: u64,
}

pub struct CreateAuctionEvent {
    auction_id: u64,
    bid_asset: AuctionAsset,
    sell_asset: AuctionAsset,
}

pub struct BidEvent {
    amount: u64,
    auction_id: u64,
    user: Identity,
}

pub struct WithdrawEvent {
    asset: AuctionAsset,
    auction_id: u64,
    user: Identity,
}
