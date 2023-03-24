use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum RandomMinterError {
    #[error("PreloadNumberIsInvalid")]
    PreloadNumberIsInvalid {},
    #[error("InvalidRoyalty")]
    InvalidRoyalty {},
    #[error("PreloadIsCompleteOrMintingStarted")]
    PreloadIsCompleteOrMintingStarted {},
    #[error("YouSentTooManyNftsToPreload")]
    YouSentTooManyNftsToPreload {},
    #[error("CannotMintYet")]
    CannotMintYet {},
    #[error("YouCantMintThisAmountOfNfts")]
    YouCantMintThisAmountOfNfts {},
    #[error("ErrorPreloadingTheSameTokenTwice")]
    ErrorPreloadingTheSameTokenTwice {},
    #[error("MaxMintPerWalletReached")]
    MaxMintPerWalletReached {},
    #[error("ExceedingMaxMintPerTxn")]
    ExceedingMaxMintPerTxn {},
}

#[derive(Debug, Error, PartialEq)]
pub enum NftMarketplaceError {
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
    #[error("ThisProfileDoesNotExist")]
    ThisProfileDoesNotExist {},
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
    #[error("InvalidCw20MessageReceived")]
    InvalidCw20MessageReceived {},
    #[error("InvalidFundsReceived")]
    InvalidFundsReceived {},
}

#[derive(Debug, Error, PartialEq)]
pub enum PriceOracleError {
    #[error("SomeDenomsAreMissingInYourUpdate")]
    SomeDenomsAreMissingInYourUpdate {},
    #[error("InvalidTimeForPriceReceived")]
    InvalidTimeForPrice {}
}

#[derive(Debug, Error, PartialEq)]
pub enum NftCollectionError {
    #[error("NoNftsMintedForThisContract")]
    NoNftsMintedForThisContract {},
}

#[derive(Debug, Error, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    Generic(GenericError),
    #[error("{0}")]
    NftMarketplace(NftMarketplaceError),
    #[error("{0}")]
    NftCollection(NftCollectionError),
    #[error("{0}")]
    PriceOracle(PriceOracleError),
    #[error("{0}")]
    RandomMinterError(RandomMinterError),
}
