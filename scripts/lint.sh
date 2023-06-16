#!/bin/bash

folders=("contracts/chihuahua-nft-marketplace" "contracts/oracle" "packages/general-utils" "packages/nft-marketplace-utils" "packages/price-oracle-utils")
base_dir=$(pwd)

for folder in "${folders[@]}"
do
    dir="$base_dir/$folder"
    if [ -d "$dir" ]; then
        cd "$dir"
        if [ -f "Cargo.toml" ]; then
            cargo clippy
        else
            echo "Error: 'Cargo.toml' not found in $dir"
        fi
        cd "$base_dir"
    else
        echo "Error: Directory $dir not found"
    fi
done
