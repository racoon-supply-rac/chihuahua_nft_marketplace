use cosmwasm_std::{Response, Timestamp};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct ResponseHandler {
    pub response: Response,
}

impl ResponseHandler {
    pub fn init_response() -> Self {
        Self {
            response: Response::new().add_attribute("action", "Instantiate: Price Oracle contract")
        }
    }

    pub fn update_config() -> Self {
        Self {
            response: Response::new().add_attribute("action", "Admin update: Update configs")
        }
    }

    pub fn log_add_new_oracle_prices(at_time: &Timestamp) -> Self {
        Self {
            response: Response::new()
                .add_attribute("action", "Execute: Add new oracle prices")
                .add_attribute("Prices at time", at_time.to_string())
        }
    }
}