# Chihuahua NFT Marketplace contract - Beta Version
This repo contains all the necessary to deploy the Chihuahua NFT Marketplace Beta Version on a CosmWasm enabled blockchain.
The code has not been audited and currently contains 98% of the codebase (but not entirely optimized).

The final version of the code will be published at the end of the project.

If you want to launch your NFTs, please make sure to use the [cw2981-multiroyalties](contracts/cw2981-multiroyalties) contracts as it is the only one compatible with the marketplace for now.

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
