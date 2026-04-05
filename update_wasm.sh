#!/bin/bash
# Build the WASM package
wasm-pack build --target web

# Move contents to docs folder (flat structure)
mkdir -p docs
mv pkg/* docs/
rm -rf pkg

# Cache busting: append timestamp to the WASM and JS imports in index.html
# This looks for .wasm and .js files and appends ?v=TIMESTAMP
TIMESTAMP=$(date +%s)
sed -i "s/\.wasm/\.wasm?v=$TIMESTAMP/g" docs/index.html
sed -i "s/\.js/\.js?v=$TIMESTAMP/g" docs/index.html

echo "WASM updated and cache busting applied to docs/index.html"
