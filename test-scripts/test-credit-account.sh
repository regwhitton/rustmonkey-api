#!/bin/bash

source common.sh-source
start_test "Credit account"

curl -s ${RUSTMONKEY_URL}/account \
    -X POST \
    -H 'Content-Type: application/json' \
    --data-binary '{"accountId":"bert","balance":10}' \
    || setup_failed

IFS="|" read HTTP_BODY HTTP_CODE <<< $(
    curl -s ${RUSTMONKEY_URL}/account/bert/balance \
        -X POST \
        -H 'Content-Type: application/json' \
        --data-binary '{"amount":0.10}' \
	    --write-out '|%{http_code}' )

assert_code 200 $HTTP_CODE
assert_body '{"balance":"10.1"}' $HTTP_BODY

end_test
