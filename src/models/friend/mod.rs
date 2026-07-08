pub mod friend;
pub mod payload;

pub use friend::{Boop, FriendList, FriendRequest, FriendStatus, FriendSummary};
pub use payload::{BoopCreate, FriendRequestDelete, FriendRequestSend};
