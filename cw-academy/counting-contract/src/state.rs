use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::Item;

// blockchain is just a key-value database
// in this case, access key is 'counter'
pub const COUNTER: Item<u64> = Item::new("counter");
pub const MINIMAL_DONATION: Item<Coin> = Item::new("minimal_donation");
pub const OWNER: Item<Addr> = Item::new("owner");
