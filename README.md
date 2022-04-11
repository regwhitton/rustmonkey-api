# rustmonkey-api 

This is an example AWS Lambda using:
* [AWS SDK for Rust](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/welcome.html) for development,
* [DynamoDB](https://docs.rs/aws-sdk-dynamodb/latest/aws_sdk_dynamodb/) for storage,
* [AWS SAM](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/what-is-sam.html) for deployment,
* [Localstack](https://docs.localstack.cloud/aws/feature-coverage/) for local testing.

The instructions and makefiles have been put together for use with Linux.  They may work with some adjustment under MacOS or WSL2.

I am a novice at Rust and this was a learning exercise.  What you find here may not be best practice.

## Setup

* AWS SAM - follow [Installing the AWS SAM CLI](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-sam-cli-install.html).
* Rustc, rustup and cargo - follow [Install Rust](https://www.rust-lang.org/tools/install).
* The musl replacement for libc (which is used by the AWS SDK for Rust):

        sudo apt install musl-tools
        rustup target add x86_64-unknown-linux-musl

* Docker and docker-compose (for use with Localstack). Probably best installed with your distribution's package manager.  Otherwise try following [Install Docker Engine](https://docs.docker.com/engine/install/) (you will not need Docker Desktop)
* Localstack command line interfaces:
    * [awslocal](https://docs.localstack.cloud/integrations/aws-cli/)
    * [samlocal](https://docs.localstack.cloud/integrations/aws-sam/)

You may also need to install python3 and python-is-python3, and add `~/.local/bin` to your PATH.

## Project Structure

### Separate Rust sub-project and target-dir

All the Rust code is placed into a sub-directory called lambda, but is built into a "target" directory directly under this.  To achieve this the project uses the CARGO\_TARGET\_DIR environment variable and the target-dir setting in lambda/.cargo/config.toml.

Every invocation of "sam build" copies all the "sources" from the project into a fresh build area in /tmp prior to every build.  Unfortunately for a custom lambda this currently means everything within the project except for ".aws-sam" and ".git".  If the target-dir is within the Rust project, then all the previously built dependency binaries will get copied as well.  This would make for a very slow build.

## Make targets

Make has been set up with targets for most development operations.  Type `make` within the project directory to see help on the available targets.  See the `Makefile` for how these targets work.

### Develop

To build and package the lambda for deployment type:

    make package

### Deploy

For the first deployment use `sam deploy --guided` to deploy the lambda into your AWS account (as shown in [Deploy your application to the AWS Cloud](https://docs.aws.amazon.com/serverless-application-model/latest/developerguide/serverless-getting-started-hello-world.html#serverless-getting-started-hello-world-deploy)).  Use the stack name `rustmonkey-api`.

For subsequent deployments or updates use:

    make deloy

### Local deploy and test

With `make local-deploy` the lambda is deployed locally using Localstack, then the tests with `test-scripts` can be used against it.  The contents of the DynamoDB table can be examined by browsing to <http://localhost:8081>.

With `make local-test` the lambda is deployed locally and the tests run automatically.

## Visual Studio Code (optional)

I have been using Visual Studio Code (from <https://code.visualstudio.com/docs/setup/linux>) with the plugins:
* rust-analyzer by matklad.
* CodeLLDB by Vadim Chugunov

You can run and debug unit tests from within Visual Studio Code.

### Debugging lambda on Localstack

Debugging the lambda it somewhat limited.  The reasons are unclear but it may be an issue with asynchronous nature of Tokio. Once a breakpoint has been hit, variables can be examined, but continuing seems to make the debugger detach.  However, you can reattach and the request is restarted.

Add to following launch configuration to .vscode/launch.json:

    {
        "name": "Attach to Rust process",
        "type": "lldb",
        "request": "custom",
        "targetCreateCommands": ["target create ${workspaceFolder}/target/x86_64-unknown-linux-musl/debug/bootstrap"],
        "processCreateCommands": ["gdb-remote 7737"],
        "sourceLanguages": ["rust"],
    }

* Run `make deploy-debug` to build and deploy for debugging.
* Run one of the tests in `test-scripts` - this will create an instance of the lambda that will stop and wait for the debugger to attach.
* From the debug panel in Visual Studio Code attach to the lambda using the configuration above.

