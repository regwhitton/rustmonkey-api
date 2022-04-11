#!/bin/bash

source common.sh-source
start_test "Dedit account"

curl -s ${RUSTMONKEY_URL}/account \
    -X POST \
    -H 'Content-Type: application/json' \
    --data-binary '{"accountId":"jim","balance":10}' \
    || setup_failed

IFS="|" read HTTP_BODY HTTP_CODE <<< $(
    curl -s ${RUSTMONKEY_URL}/account/jim/balance \
        -X POST \
        -H 'Content-Type: application/json' \
        --data-binary '{"amount":-0.02}' \
	    --write-out '|%{http_code}' )

assert_code 200 $HTTP_CODE
assert_body '{"balance":"9.98"}' $HTTP_BODY

end_test
