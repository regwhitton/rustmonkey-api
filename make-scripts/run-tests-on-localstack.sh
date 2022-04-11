#!/bin/bash
#
# Start Localstack, deploy stack onto it, run tests and stop Localstack.
#

rm -f target/localstack.ready

echo "Starting Localstack ..."
env AWS_REGION=${AWS_REGION} docker-compose -f localstack/deployment-compose.yml up --detach localstack
trap "env AWS_REGION=${AWS_REGION} docker-compose -f "$PWD"/localstack/deployment-compose.yml down" EXIT

echo "Waiting until Localstack ready ..."
while [ ! -f target/localstack.ready ]
do
    sleep 1
done
echo "Localstack ready!"

echo "Deploying stack onto Localstack ..."
samlocal deploy --stack-name rustmonkey-api --resolve-s3 --region "${AWS_REGION}" --no-confirm-changeset || exit 1
echo "Stack deployed!"


echo "Running tests ..."
cd ./test-scripts/
./all-tests.sh

