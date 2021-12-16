
#
# "sam build" copies sources (from lambda) to a fresh build area in /tmp.
# (Configured by CodeUri in template.yaml)
# To avoid the copy taking all the previously built dependencies
# binaries and taking a long time, we keep these outside of lambda.
# Note use of CARGO_TARGET_DIR.
#
# If you use Visual Studio Code with the rust-analyzer plugin then
# to do the same: add the following option to .vscode/setting.json
#  "rust-analyzer.server.extraEnv": { "CARGO_TARGET_DIR":"../target" }
#

#
# Default "make" used for compiling.  Doesn't hide compiler output.
#
.PHONY: build
build:
	cd lambda && env PROFILE=debug CARGO_TARGET_DIR="$(PWD)/target" $(MAKE) build

#
# Compile is release mode. Doesn't hide compiler output.
#
.PHONY: build-release
build-release:
	cd lambda && env PROFILE=release CARGO_TARGET_DIR="$(PWD)/target" $(MAKE) build

#
# Run a debug profile "sam build".  Needed for "make start-api" and local debugging.
#
.PHONY: sam-build
sam-build:
	env PROFILE=debug CARGO_TARGET_DIR="$(PWD)/target" sam build

#
# Run a release profile "sam build".  Create optimized exe for deployment.
#
.PHONY: sam-build-release
sam-build-release:
	env PROFILE=release CARGO_TARGET_DIR="$(PWD)/target" sam build

#
# Check changes made to the template.yaml
#
.PHONY: sam-validate
sam-validate:
	sam validate -t template.yaml

#
# Deploy or update the lambda onto AWS and create or update the dynamodb tables.
#
.PHONY: sam-deploy
sam-deploy:
	sam deploy

#
# Remove stack (Lambdas, DynamoDb tables, S3 folders etc)
#
.PHONY: sam-delete
sam-delete:
	sam delete

#
# Remove all the compiled binaries.
#
.PHONY: clean
clean:
	rm -rf target .aws-sam

#
# Start dynamodb-local and dynamodb-admin for local use.
# For dynamodb-admin: browse to http://localhost:8001 
#
.PHONY: start-db
start-db:
	docker-compose -f db/docker-compose.yml pull
	docker-compose -f db/docker-compose.yml up

#
# Start the lambda locally.
#
.PHONY: start-api
start-api:
	sam local start-api --docker-network db_db-network --warm-containers EAGER --env-vars "local-vars.json"
