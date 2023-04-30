set -ux

source ~/.wasmd/defaults.env
export NODE="--node $RPC"
export TXFLAG="${NODE} --chain-id ${CHAIN_ID} --gas-prices 0.25${FEE_DENOM} --gas auto --gas-adjustment 1.3"

# STEP 1) storing wasm contract
# wasm store
wasm_path="./wasm.d/cw_nameservice"
wasmd tx wasm store $wasm_path \
    --from wallet \
    --chain-id ${CHAIN_ID} \
    --gas-prices 0.25${FEE_DENOM} \
    --gas auto \
    --gas-adjustment 1.3 \
    --node https://rpc.malaga-420.cosmwasm.com:443 \
    --output json -b block


# STEP 2) querying your stored wasm code
CODE_ID=$(cat store_res| jq -r '.logs[0].events[-1].attributes[0].value')
wasmd query wasm list-contract-by-code $CODE_ID --output json


# STEP 3)
# you can also download the wasm from the chain and check that the diff between them is empty
CODE_ID=$(cat store_res| jq -r '.logs[0].events[-1].attributes[0].value')
wasmd query wasm code $CODE_ID $NODE download.wasm

# STEP 4) instantiate your contract
CODE_ID=$(cat store_res| jq -r '.logs[0].events[-1].attributes[0].value')
INIT='{"purchase_price":{"amount":"100","denom":"upebble"},"transfer_price":{"amount":"999","denom":"upebble"}}'
NODE="--node $RPC"
wasmd tx wasm instantiate "$CODE_ID" "$INIT" \
    --from wallet \
    --no-admin \
    --label "awesome name service" \
    --chain-id ${CHAIN_ID} \
    --gas-prices 0.25${FEE_DENOM} \
    --gas auto \
    --gas-adjustment 1.3

# check the contract state (and account balance)
# wasmd query wasm list-contract-by-code $CODE_ID $NODE --output json
wasmd query wasm list-contract-by-code $CODE_ID  -o json | jq .
CONTRACT=$(wasmd query wasm list-contract-by-code $CODE_ID --output json | jq -r '.contracts[-1]')
echo $CONTRACT

# contract state
wasmd query wasm contract $CONTRACT -o json | jq .
wasmd query wasm contract-state all $CONTRACT -o json  | jq .

# "key": "636F6E666967",
wasmd query wasm contract-state raw $CONTRACT 636F6E666967

# key encoding and decoding
'''
let config = Config {
        purchase_price: msg.purchase_price,
        transfer_price: msg.transfer_price,
    };
'''
echo -n config | xxd -ps

wasmd query wasm contract-state all $CONTRACT -o json | jq -r '.models[0].key' | xxd -r -ps
wasmd query wasm contract-state all $CONTRACT --output "json" | jq -r '.models[0].value' | base64 -d


# execute fails if wrong person
REGISTER='{"register":{"name":"fred"}}'
wasmd tx wasm execute $CONTRACT "$REGISTER" \
    --amount 1000000umlg \
    --from wallet \
    --chain-id ${CHAIN_ID} \
    --gas-prices 0.5${FEE_DENOM} \
    --gas auto \
    --gas-adjustment 1.3


