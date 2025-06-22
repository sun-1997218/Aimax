/// Module for the initialize instruction
pub mod initialize;
/// Module for the withdraw_tokens instruction
pub mod withdraw_tokens;
/// Module for the ccip_receive instruction
pub mod ccip_receivers;
/// Module for the get_latest_message instruction
pub mod get_latest_message;

// 将各个模块的处理函数导出到顶层
pub use initialize::handler as initialize_handler;
pub use withdraw_tokens::handler as withdraw_tokens_handler;
pub use ccip_receivers::handler as ccip_receive_handler;
pub use get_latest_message::handler as get_latest_message_handler;