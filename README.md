# psi-sample

PSI Test tool is an open source tool to assist web developers that runs Page Speed Insight test manually!

## Installing

To install the psi-test tool, run inside the terminal:

```sh
cargo install psi-test
```

If you don't have Cargo package manager for Rust install it. For more information about installation https://doc.rust-lang.org/cargo/getting-started/installation.html

## Using PSI-Test Tool
> :warning: get the google page speed insight API token here: https://developers.google.com/speed/docs/insights/v5/get-started#APIKey

Examples of how to run psi-test tool

### Default
Using the default number-of-runs that is 20.

```sh
psi-test --token=<<your_token>> <<page_url>>
```

### Passing number-of-runs flag

```sh
psi-test --token=<<your_token>> --number-of-values=10 <<page_url>>
```

For more information run:

```sh
psi-test --help
```
