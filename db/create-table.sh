#!/bin/bash

export AWS_ACCESS_KEY_ID=local_access_id
export AWS_SECRET_ACCESS_KEY=local_access_key
export LOCAL_DYNAMODB_ENDPOINT=http://localhost:8000 

set -x 

aws dynamodb delete-table \
	--table-name Accounts \
	--endpoint-url $LOCAL_DYNAMODB_ENDPOINT \
    --no-cli-pager

aws dynamodb create-table \
    --table-name Accounts \
    --attribute-definitions AttributeName=accountId,AttributeType=S \
    --key-schema AttributeName=accountId,KeyType=HASH \
    --provisioned-throughput ReadCapacityUnits=1,WriteCapacityUnits=1 \
    --endpoint-url $LOCAL_DYNAMODB_ENDPOINT \
    --no-cli-pager 

#&& aws dynamodb put-item \
#    --table-name Accounts \
#    --item '{"accountId":{"S":"sid"},"balance":{"N":"0.00"}}' \
#    --endpoint-url $LOCAL_DYNAMODB_ENDPOINT \
#    --no-cli-pager

