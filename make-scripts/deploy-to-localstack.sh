#!/bin/bash
#
# Start Localstack (and DynamoDB-admin), deploy stack onto it, then wait
#

rm -f target/localstack.ready

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
    echo "Stack deployed! Ready for test."
)&

echo "Starting Localstack ..."
trap "env AWS_REGION=${AWS_REGION} docker-compose -f "$PWD"/localstack/deployment-compose.yml down" EXIT
env AWS_REGION=${AWS_REGION} docker-compose -f localstack/deployment-compose.yml up
