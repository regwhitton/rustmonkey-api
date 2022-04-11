#!/bin/bash

source common.sh-source
start_test "Query account"

curl -s ${RUSTMONKEY_URL}/account \
    -X POST \
    -H 'Content-Type: application/json' \
    --data-binary '{"accountId":"john","balance":50.22}' \
    || setup_failed

IFS="|" read HTTP_BODY HTTP_CODE <<< $(
    curl -s ${RUSTMONKEY_URL}/account/john \
        --write-out '|%{http_code}' )

assert_code 200 $HTTP_CODE
assert_body '{"accountId":"john","balance":"50.22"}' $HTTP_BODY

end_test
