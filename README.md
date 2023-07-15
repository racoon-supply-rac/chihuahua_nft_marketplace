# Chihuahua NFT Marketplace contract
This repo contains all the necessary to deploy the Chihuahua NFT Marketplace Beta Version on a CosmWasm enabled blockchain.
The code has not been audited.

If you want to launch your NFTs, please make sure to use the following NFT standard [cw2981-multiroyalties](contracts/cw2981-multiroyalties) as it is the only one compatible with the marketplace for now.

## Structure
The **packages** contain most of the structs and helpers needed by the contracts and the **contracts** works together
to make the NFT marketplace.

## Tests
To launch all the tests, just use: `bash scripts/tests.sh`.

## Linting
`bash scripts/lint.sh`

## Schema
`bash scripts/schema.sh`

## Optimized build
Build your wasm files: `bash scripts/optimized_build.sh`.
