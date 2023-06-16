use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct RoyaltyInfo {
    pub address: String,
    pub thousands: String,
}

#[cw_serde]
pub struct Trait {
    pub display_type: Option<String>,
    pub trait_type: String,
    pub value: String,
}

#[cw_serde]
pub struct Cw2981LegacyMetadata {
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Vec<Trait>>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
    pub royalty_info: Option<Vec<RoyaltyInfo>>,
}
