# AWS CFN resource picker

Pick a stack then a resource. The output is, **for the mapped resources**, a path which one can use in AWS console or (as per my use-case) in combination with [`aws-sso-util`](https://github.com/benkehoe/aws-sso-util).

https://user-images.githubusercontent.com/26322927/190849528-7234a95e-5fa6-4a57-8d62-d67a918aabb9.mov

Please note that I have no idea what I'm doing when it comes to Rust programming, so the code is probably suboptimal 🤷‍♂️.

## Usage

1. Make sure you have the AWS profile configured in your shell.

2. Run the following command.

   ```shell
   cargo run
   ```

## Learnings

- It seems like there is no good way to mock a given AWS SDK client itself. In Go, you the library exposes an `interface` which has all the methods. That is not the case for rust

  - Instead of providing a mock AWS Client, people advocate for [mocking the credentials and HTTP layer](https://github.com/awslabs/aws-sdk-rust/issues/199#issuecomment-904558631).

- **The CloudFormation CLI/SDK `list-stacks` command includes stacks that were deleted (are not visible in the console)**.

  - This is very surprising, I thought I had a bug in my application!

  - And, in the AWS fashion, **there is no easy way to list all stacks except the ones that were already deleted**.
    You have to filter on ALL statuses except the `DELETE_COMPLETE` one. LOL!

- The `Try` and `Into` traits are pretty awesome.

  - What is not so awesome is that you cannot extend the "built-in" structures with your custom implementation of `Try` or `Into`. For example, `impl Into<String> for Vec<usize>` is not possible.
