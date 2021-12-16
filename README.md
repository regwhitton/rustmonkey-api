# rustmonkey-api 

This is an example a Lambda using the [AWS SDK for Rust](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html), [DynamoDB](https://docs.rs/aws-sdk-dynamodb/latest/aws_sdk_dynamodb/) and [AWS SAM](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/what-is-sam.html).

The instructions and makefiles here assume use under Linux.  They may work with some adjustment under MacOS or WSL2.

I am a novice at Rust and this was a learning exercise.  What you find here may not be best practice.

## Installations

Install AWS SAM as per https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-install.html

Install Rust (including rustup and cargo) using instructions at https://www.rust-lang.org/tools/install

To build the project for the `x86_64-unknown-linux-musl` target environment that it will deployed into, you will also need to install:
* musl-tools (on Linux: `sudo apt install musl-tools`)
* the pre-built `std` Rust package for musl. Use `rustup target add x86_64-unknown-linux-musl`

To run your Lambda locally [Install Docker community edition](https://hub.docker.com/search/?type=edition&offering=community)

### Visual Studio Code (optional)

I have been using Visual Studio Code (from https://code.visualstudio.com/docs/setup/linux) with the rust-analyzer plugin by matklad.

I haven't yet figured out how to debug within the SAM container.

#### .vscode/settings

`sam build` copies everything from the CodeUri folder (lambda) to a temporary build area in /tmp.
To avoid it spending a lot of time uselessly copying previously built dependencies, the
target folder is configured to be outside of lambda by setting the CARGO_TARGET_DIR environment variable.

If you use Visual Studio Code with the rust-analyzer plugin then you must do the same by adding this option
.vscode/setting.json

    {
        ...
        "rust-analyzer.server.extraEnv": { "CARGO_TARGET_DIR":"../target" }
    }

## Develop

Use `make clean` to remove anything previously built.

Use `make` to compile.

Use `make sam-build` to compile and create AWS SAM deployable.

Use `make start-db` to start up local DynamoDB in Docker container.

Use `./db/create-table.sh` to create local DynamoDB table.

Use `make start-api` to start up the lamda and http gateway locally, then `curl http://localhost:3000/`.

Use the scripts within `test-scripts` to try out the functionality by making curl requests to the local http gateway.  Have fun trying to debit an account without sufficient credit.

Browse to <http://localhost:8001> to view the contents of the DynamoDB table.

Also: check out the other commands in the Makefile file.

## Deploy

For the first deployment use `sam deploy --guided` to deploy the lambda into your AWS account (as shown in [Deploy your application to the AWS Cloud](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-getting-started-hello-world.html#serverless-getting-started-hello-world-deploy)).  I suggest using the stack name `rustmonkey-api`.

For subsequent deployments or updates you can use `sam deploy` or `make sam-deploy` and these will re-use the previous answers.

Clean up your AWS resources by running `make sam-delete`. (SAM creates an S3 bucket during deployment and logs that are not removed)


