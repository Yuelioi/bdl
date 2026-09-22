//! 消息

mod client;
pub mod msg;
pub mod params;
pub mod private_msg;
pub mod private_msg_content;
pub mod settings;

pub use client::MessageClient;
pub use params::{
    MessageReplyFeedParams, MessageSingleUnreadParams, MessageUnreadCountParams, SingleUnreadType,
};
pub use private_msg::MessageSendParams;
