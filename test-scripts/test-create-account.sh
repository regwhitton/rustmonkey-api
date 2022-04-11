#!/bin/bash

source common.sh-source
start_test "Create account"

HTTP_CODE=$(
    curl -s ${RUSTMONKEY_URL}/account \
        -X POST \
        -H 'Content-Type: application/json' \
        --data-binary '{"accountId":"sid","balance":0}' \
	    --write-out '%{http_code}' )

assert_code 201 $HTTP_CODE

end_test
