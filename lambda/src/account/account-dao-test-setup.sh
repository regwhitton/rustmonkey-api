#!/bin/bash
#
# Script used by account-dao unit test to start-up DynamoDB-local,
# create Accounts table, populate it with test data, signal setup complete
# and wait for signal that test is finished.
#

# Could be passed through the env from the test.
export AWS_ACCESS_KEY_ID=local_access_id
export AWS_SECRET_ACCESS_KEY=local_access_key
export AWS_DEFAULT_REGION=eu-west-2

LOG=../target/account-dao-test.log
echo "==> Script begins - $(date)" > $LOG

#
# Functions used by main body of script below.
#

function find_free_port () {
	#echo 8000
    port=8500
    isfree=$(netstat -taln | grep $port)
    while [[ -n "$isfree" ]]; do
        port=$[port+1]
        isfree=$(netstat -taln | grep $port)
    done
    echo $port
}

function start_dynamodb_on_port () {
    port=$1
    
    if [ -z $(docker images -q amazon/dynamodb-local:latest) ] 
    then
        docker pull amazon/dynamodb-local:latest
    fi

    echo "==> Starting DynamoDB container ..." >>$LOG
    docker run --rm -p $port:8000 amazon/dynamodb-local:latest >>$LOG 2>&1 &
    DOCKER_PID=$!
    trap "kill $DOCKER_PID" EXIT
}

function wait_until_dynamo_db_ready () {
    ENDPOINT=$1
    attempts=60
    found=0
    echo "==> Waiting for DynamoDB to start ..." >>$LOG
    while [[ attempts-- -ne 0 ]]
    do
        if aws dynamodb wait table-not-exists --table-name NotExistantTable --endpoint-url $ENDPOINT >>$LOG 2>&1
        then
            found=1
            break
        fi
        sleep 0.5
    done
    if [[ $found -eq 0 ]]
    then
        echo "==> DynamoDB failed to start" >>$LOG
        exit
    else
        echo "==> DynamoDB started" >>$LOG
    fi
}

function wait_until_dynamodb_table_exists () {
    ENDPOINT=$1
    table_name="$2"
    attempts=60
    found=0
    echo "==> Waiting for table $table_name to exist ..." >>$LOG
    while [[ attempts-- -ne 0 ]]
    do
        if aws dynamodb wait table-exists --table-name "$table_name" --endpoint-url $ENDPOINT >>$LOG 2>&1
        then
            found=1
            break
        fi
        sleep 0.5
    done
    if [[ $found -eq 0 ]]
    then
        echo "==> table $table_name not created" >>$LOG
        exit
    else
        echo "==> table $table_name created." >>$LOG
    fi
}

function wait_until_input_stream_closes() {
	while read line
    do
        echo "==> input: $line" >>$LOG
    done
}

#
# Main body of script
#

port=$(find_free_port)
ENDPOINT=http://localhost:$port/

start_dynamodb_on_port $port

wait_until_dynamo_db_ready $ENDPOINT

aws dynamodb create-table \
    --table-name Accounts \
    --attribute-definitions AttributeName=accountId,AttributeType=S \
    --key-schema AttributeName=accountId,KeyType=HASH \
    --provisioned-throughput ReadCapacityUnits=1,WriteCapacityUnits=1 \
    --endpoint-url $ENDPOINT \
    --no-cli-pager >> $LOG 2>&1

wait_until_dynamodb_table_exists $ENDPOINT Accounts

# Signal to parent that database is ready.
echo "==> Ready marker and endpoint ($ENDPOINT) written to output" >> $LOG
echo READY $ENDPOINT

wait_until_input_stream_closes

echo "==> Input stream closed - $(date)" >> $LOG
sleep 20
echo "==> exit - $(date)" >> $LOG

