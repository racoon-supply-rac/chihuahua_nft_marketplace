# Chihuahua NFT Marketplace contract - Alpha Version
This repo contains all the necessary to deploy the Chihuahua NFT Marketplace on a CosmWasm enabled blockchain.
The code has not been audited and only contains 55% of the features of the final marketplace.
If you want to use the code, please refer to the `LICENSE.MIT` file.

The beta version containing 90-95% of the features will be released by the end of April 2023 or early May 2023. 
For more info about the milestones, visit [this repo](https://github.com/racoon-supply-rac/chihuahua_nft_marketplace_docs).

If you want to launch your NFTs, please make sure to use the [cw2981-multiroyalties](contracts/cw2981-multiroyalties) contracts as it is the only one compatible with the marketplace for now.

## Structure
The **packages** contain most of the structs and helpers needed by the contracts and the **contracts** works together
to make the NFT marketplace.

## Tests
To launch all the tests, just use: `bash tests.sh`.

## Optimized build
Build your wasm files: `bash optimized_build.sh`.
