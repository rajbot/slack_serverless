.PHONY: build test check fmt clippy clean doc lambda-build lambda-deploy

# Default target
all: check test build

# Build the project
build:
	cargo build

# Build in release mode
build-release:
	cargo build --release

# Run tests
test:
	cargo test

# Check the project (fast compile check)
check:
	cargo check

# Format code
fmt:
	cargo fmt

# Run clippy for additional linting
clippy:
	cargo clippy -- -D warnings

# Clean build artifacts
clean:
	cargo clean

# Generate documentation
doc:
	cargo doc --no-deps --open

# Build for AWS Lambda (requires cargo-lambda)
lambda-build:
	cargo lambda build --release

# Deploy to AWS Lambda (requires cargo-lambda and AWS credentials)
lambda-deploy: lambda-build
	cargo lambda deploy

# Run a specific example
run-example:
	@echo "Available examples:"
	@echo "  make run-basic    - Run basic_app example"
	@echo "  make run-oauth    - Run oauth_app example"
	@echo "  make run-lambda   - Run lambda_deployment example"

run-basic:
	cargo run --example basic_app

run-oauth:
	cargo run --example oauth_app --features oauth

run-lambda:
	cargo run --example lambda_deployment --features lambda

# Setup for development
setup:
	@echo "Setting up development environment..."
	rustup component add rustfmt clippy
	@echo "Consider installing cargo-lambda for AWS Lambda deployment:"
	@echo "  pip3 install cargo-lambda"

# Check all features
check-all:
	cargo check --all-features
	cargo check --no-default-features
	cargo check --features oauth
	cargo check --features lambda

# CI/CD pipeline simulation
ci: fmt clippy check-all test build-release
	@echo "✅ All CI checks passed!"