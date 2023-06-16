#[cfg(not(feature = "library"))]
pub mod contract;
pub mod execute;
pub mod msg;
pub mod query;
pub mod state;
pub mod tests;
