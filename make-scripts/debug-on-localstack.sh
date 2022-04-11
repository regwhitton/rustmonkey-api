#!/bin/bash
#
# Start Localstack, deploy debuggable stack onto it.
#

rm -f target/localstack.ready

echo "Creating debuggable lambda image ..."
if [ -z $(docker images -q local_lambci:provided) ] 
then
    docker build -t local_lambci:provided localstack/debugging-lambda || exit 1
fi

(
    # In background
    echo "Waiting until Localstack ready ..."
    while [ ! -f target/localstack.ready ]
    do
        sleep 1
    done
    echo "Localstack ready!"

    echo "Deploying stack onto Localstack ..."
    samlocal deploy --stack-name rustmonkey-api --resolve-s3 --region "${AWS_REGION}" --no-confirm-changeset || exit 1

    echo "Stack deployed! You need to invoke API and attach debugger."
)&

function stop_lambda_container_and_localstack() {
    CONTAINER_ID="$(docker ps -q -f NAME=localstack_lambda_arn)"
    if [ ! -z "$CONTAINER_ID" ]
    then
        docker stop $CONTAINER_ID
    fi
    env AWS_REGION=${AWS_REGION} \
        docker-compose -f "$PWD"/localstack/debugging-compose.yml down
}
trap 'stop_lambda_container_and_localstack' EXIT

echo "Starting Localstack ..."
env AWS_REGION=${AWS_REGION} docker-compose -f localstack/debugging-compose.yml up
