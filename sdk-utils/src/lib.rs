mod blueprint;
mod command;
pub mod proto_types;
mod storage;

pub use blueprint::*;
pub use command::*;
pub use storage::*;

pub fn compute_signal_length(max_length: usize) -> usize {
    (max_length / 31) + if max_length % 31 != 0 { 1 } else { 0 }
}
