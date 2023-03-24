#[cfg(not(feature = "library"))]
pub mod contract;
pub mod execute;
pub mod query;
pub mod state;
pub mod msg;
pub mod tests;
pub mod integration_tests;
