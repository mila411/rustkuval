# RustKuval: Kubernetes YAML Validator

⚠️⚠️⚠️<br>
This is a toy I created to play with Rust and kubernetes. Please note that it lacks many features that are Production Ready<br>
⚠️⚠️⚠️

RustKuval is a lightweight Kubernetes YAML validation tool written in Rust. It parses Kubernetes resource YAML files, validates them against Kubernetes OpenAPI schemas, and reports errors related to missing required fields.

## Features

- **Kubernetes OpenAPI schema validation:** Fetches the OpenAPI schema dynamically and validates YAML files against it.
- **Folder support:** Validates all YAML files in a folder asynchronously.
- **Error filtering:** Only reports errors related to missing required fields.
- **Dynamic validation:** No hardcoding of resource types; the validator adapts dynamically based on the schema.
- **Output:**
  - Displays apiVersion and kind for each resource.
  - Indicates whether validation succeeded or failed.

## Features I want to add in the future

- CRD Verification

## Installation

Clone this repository:

```sh
git clone https://github.com/your_username/rustkuval.git
cd rustkuval
```

Build the project:

```sh
cargo build --release
```

Run the program:

```sh
cargo run -- <path_to_yaml_or_folder>
```

## Usage

### Validate a Single File

To validate a single Kubernetes YAML file, provide the file path as an argument:

```sh
cargo run -- ./example.yaml
```

### Validate All Files in a Folder

```sh
cargo run -- ./yaml_folder
```

### Expected Output

For each file, the program outputs:

1. The detected apiVersion and kind.
2. A success message if validation passes.
3. Error messages for missing required fields.

### Example Output

```sh
File: example.yaml
Detected: apiVersion=v1, kind=Pod
Validation successful!

File: invalid.yaml
Detected: apiVersion=v1, kind=Pod
Validation failed!
Missing required field: metadata.name
```

## Requirements

- **Rust:** Version 1.65 or higher.
- **Internet Access:** Required to fetch Kubernetes OpenAPI schemas.

## How It Works

1. **Fetch OpenAPI Schema:** On the first run, the program retrieves the Kubernetes OpenAPI schema via HTTP and caches it for subsequent use.
2. **Parse YAML Files:** Each YAML file is parsed into JSON format for validation.
3. **Validate Against Schema:** The parsed content is validated dynamically against the OpenAPI schema.
4. **Output Results:** The program outputs either a success message or error details for missing required fields.
