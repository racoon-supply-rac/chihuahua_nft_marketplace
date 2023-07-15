use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum GenericError {
    #[error("DivisionError")]
    DivisionError {},
    #[error("MultiplicationError")]
    MultiplicationError {},
    #[error("ErrorWhileInstantiatingCw2981")]
    ErrorWhileInstantiatingCw2981 {},
    #[error("InvalidReplyID")]
    InvalidReplyID {},
    #[error("NotImplementedYet")]
    NotImplementedYet {},
    #[error("Unauthorized")]
    Unauthorized {},
    #[error("InvalidExecuteMessage")]
    InvalidExecuteMessage {},
    #[error("ContractDisabled")]
    ContractDisabled {},
    #[error("InvalidDenominationReceived")]
    InvalidDenominationReceived {},
    #[error("InvalidFundsReceived")]
    InvalidFundsReceived {},
    #[error("Addr ({address}) and block ({block})")]
    PendingError { address: String, block: String },
}

#[derive(Debug, Error, PartialEq)]
pub enum PriceOracleError {
    #[error("SomeDenomsAreMissingInYourUpdate")]
    SomeDenomsAreMissingInYourUpdate {},
    #[error("InvalidTimeForPriceReceived")]
    InvalidTimeForPrice {},
}

#[derive(Debug, Error, PartialEq)]
pub enum GameError {
    #[error("GameNotEnabled")]
    GameNotEnabled {},
    #[error("BetAmountNotWithinTheAcceptedRange")]
    BetAmountNotWithinTheAcceptedRange {},
    #[error("LiquidityIsTooLowForThisDenom")]
    LiquidityIsTooLowForThisDenom {},
    #[error("InvalidBetConfiguration")]
    InvalidBetConfiguration {},
    #[error("InvalidBetInput")]
    InvalidBetInput {},
    #[error("LPNotFound")]
    LPNotFound {},
    #[error("GameNotAvailableYet")]
    GameNotAvailableYet {},
    #[error("MaxUsdcPotentialExceeded")]
    MaxUsdcPotentialExceeded {},
    #[error("JobAlreadyExists")]
    JobAlreadyExists {},
    #[error("UnexpectedError")]
    UnexpectedError {},
}

#[derive(Debug, Error, PartialEq)]
pub enum SchedulerError {
    #[error("NoJobs")]
    NoJobs {},
}


#[derive(Debug, Error, PartialEq)]
pub enum NftCollectionError {
    #[error("NoNftsMintedForThisContract")]
    NoNftsMintedForThisContract {},
}

#[derive(Debug, Error, PartialEq)]
pub enum NftMarketplaceError {
    #[error("InvalidNftCollection")]
    InvalidNftCollection {},
    #[error("InvalidInput")]
    InvalidInput {},
    #[error("InvalidRoyalty")]
    InvalidRoyalty {},
    #[error("InvalidAmountReceivedForLevelUp")]
    InvalidAmountReceivedForLevelUp {},
    #[error("AlreadyLevel3")]
    AlreadyLevel3 {},
    #[error("NeedToFillAllThePerks")]
    NeedToFillAllThePerks {},
    #[error("InvalidRewards")]
    InvalidRewards {},
    #[error("InvalidLevelUp")]
    InvalidLevelUp {},
    #[error("InvalidAcceptedDenoms")]
    InvalidAcceptedDenoms {},
    #[error("InvalidMarketplaceFee")]
    InvalidMarketplaceFee {},
    #[error("NftFloorError")]
    NftFloorError {},
    #[error("NftNotForSale")]
    NftNotForSale {},
    #[error("YouDontOwnThisOffer")]
    YouDontOwnThisOffer {},
    #[error("CantUseAdditionalInfoIfNotContract")]
    CantUseAdditionalInfoIfNotContract {},
    #[error("AdditionalInfoNeedsToBeFilled")]
    AdditionalInfoNeedsToBeFilled {},
    #[error("NftCollectionAlreadyExists")]
    NftCollectionAlreadyExists {},
    #[error("Cancel your offer before making a new one")]
    OfferAlreadyExists {},
    #[error("RevokeYourApprovalBeforeCancellingSale")]
    RevokeYourApprovalBeforeCancellingSale {},
    #[error("YourProfileAlreadyExists")]
    YourProfileAlreadyExists {},
    #[error("InvalidNftShowcaseReceived")]
    InvalidNftShowcaseReceived {},
    #[error("InvalidUsername")]
    InvalidUsername {},
    #[error("ThisUsernameIsAlreadyTaken")]
    ThisUsernameIsAlreadyTaken {},
    #[error("ReceiverDoesNotExist")]
    ReceiverDoesNotExist {},
    #[error("ThisProfileDoesNotExist")]
    ThisProfileDoesNotExist {},
    #[error("MessageRecipientDoesNotExist")]
    MessageRecipientDoesNotExist {},
    #[error("InvalidMessage")]
    InvalidMessage {},
    #[error("UsernameUnexpectedError")]
    UsernameUnexpectedError {},
    #[error("InvalidSenderOrYouCantChangeProfileAddress")]
    InvalidSenderOrYouCantChangeProfileAddress {},
    #[error("This Sale Does Not Exist")]
    SaleDoesNotExist {},
    #[error("InvalidSeller")]
    InvalidSeller {},
    #[error("InvalidSellerInformation")]
    InvalidSellerInformation {},
    #[error("CantOfferOnYourOwnNft")]
    CantOfferOnYourOwnNft {},
    #[error("InvalidOffererInformation")]
    InvalidOffererInformation {},
    #[error("NftCollectionNotListed")]
    NftCollectionNotListed {},
    #[error("SaleAlreadyExists")]
    SaleAlreadyExists {},
    #[error("InvalidDenomOrValueReceivedForListingFee")]
    InvalidDenomOrValueReceivedForListingFee {},
    #[error("YouDontOwnThisTokenID")]
    YouDontOwnThisTokenID {},
    #[error("InvalidPriceForTheSale")]
    InvalidPriceForTheSale {},
    #[error("InvalidExpirationTimeForTheSale")]
    InvalidExpirationTimeForTheSale {},
    #[error("InvalidExpirationTimeForTheOffer")]
    InvalidExpirationTimeForTheOffer {},
    #[error("InvalidBuyerInformation")]
    InvalidBuyerInformation {},
    #[error("CantCancelASaleYouDontOwn")]
    CantCancelASaleYouDontOwn {},
    #[error("InvalidFundsForOffer")]
    InvalidFundsForOffer {},
    #[error("InvalidOfferDenom")]
    InvalidOfferDenom {},
    #[error("InvalidOfferValueReceived")]
    InvalidOfferValueReceived {},
    #[error("InvalidPrice")]
    InvalidPrice {},
    #[error("BuyAndSellCannotBeNoneTogether")]
    BuyAndSellCannotBeNoneTogether {},
    #[error("BuyAndSellCannotBeFilledTogether")]
    BuyAndSellCannotBeFilledTogether {},
}



#[derive(Debug, Error, PartialEq)]
pub enum ReserveError {
    #[error("InvalidInitMsg")]
    InvalidInitMsg {},
    #[error("ReserveIsTooLow")]
    ReserveIsTooLow {},
    #[error("Addr ({address}) and block ({block})")]
    ReserveTooLowPBVAL { address: String, block: String },
    #[error("Addr ({address}) and block ({block})")]
    ReserveTooLowD { address: String, block: String },
}

#[derive(Debug, Error, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    Generic(GenericError),
    #[error("{0}")]
    PriceOracle(PriceOracleError),
    #[error("{0}")]
    GameError(GameError),
    #[error("{0}")]
    ReserveError(ReserveError),
    #[error("{0}")]
    SchedulerError(SchedulerError),
    #[error("{0}")]
    NftMarketplaceError(NftMarketplaceError),
    #[error("{0}")]
    NftCollectionError(NftCollectionError),
}
