# Standout App Bridge

`app_bridge` is a Ruby gem designed to facilitate communication with WebAssembly components that implement the WIT specification `standout:app`. This gem is developed for use in Standout's products.

## Installation

Add the following line to your `Gemfile`:

```ruby
gem 'app_bridge'
```

Then, install the gem by running:

```bash
bundle install
```

## Usage

To use this gem, you need a WebAssembly component that adheres to the specification defined in `ext/app_bridge/wit/world.wit`.

You can check out the example components in `spec/fixtures/components` to see how such a component should be structured.

Once you have a WebAssembly component, you can use the gem as follows:

```ruby
require 'app_bridge'

app = AppBridge::App.new('path/to/your/component.wasm')
app.triggers # => ['trigger1', 'trigger2']
```

### File Handling

The gem provides a `file.normalize` function for handling files in connectors. It automatically detects the input format (URL, data URI, or base64) and returns normalized file data.

#### In your WASM connector (JavaScript):

```javascript
import { normalize } from 'standout:app/file@4.0.0';

// Normalize any file source - input type is auto-detected
const fileData = normalize(
  input.fileUrl,           // URL, data URI, or base64 string
  [["Authorization", token]], // Optional headers for URL requests
  "invoice.pdf"            // Optional filename override
);

// Returns: { base64, contentType, filename }

// Include in your action output
return {
  serializedOutput: JSON.stringify({
    document: fileData  // Will be replaced with blob ID by platform
  })
};
```

#### In your WASM connector (Rust):

```rust
use crate::standout::app::file::normalize;

let file_data = normalize(
    &input.file_url,
    Some(&[("Authorization".to_string(), token)]),
    Some("invoice.pdf"),
)?;

// file_data contains { base64, content_type, filename }
```

#### Output schema:

Mark file fields with `format: "file-output"` so the platform knows to process them:

```json
{
  "properties": {
    "document": {
      "type": "object",
      "format": "file-output"
    }
  }
}
```

#### Platform configuration:

Configure the file uploader in your Rails app:

```ruby
AppBridge.file_uploader = ->(file_data) {
  blob = ActiveStorage::Blob.create_and_upload!(
    io: StringIO.new(Base64.decode64(file_data['base64'])),
    filename: file_data['filename'],
    content_type: file_data['content_type']
  )
  blob.signed_id
}
```

The gem automatically replaces file data with the return value (in this example blob IDs) before returning the action response.

## Backward Compatibility

The gem supports **multi-version WIT interfaces**, allowing connectors built against older WIT versions to continue working when the gem is updated.

### How it works

When loading a WASM component, the gem automatically detects which WIT version it was built against:

1. **V4 components** (current, `standout:app@4.0.0`): Full feature support including the `file` interface
2. **V3 components** (`standout:app@3.0.0`): Legacy support without file interface

### Adding support for new WIT versions

When adding a new WIT version (e.g., v5), follow these steps:

#### 1. Create the WIT file

Copy the latest version and modify:

```bash
cp -r ext/app_bridge/wit/v4 ext/app_bridge/wit/v5
```

Edit `ext/app_bridge/wit/v5/world.wit`:
- Update the package version: `package standout:app@5.0.0;`
- Add new interfaces or modify existing ones

#### 2. Add the bindgen module

In `ext/app_bridge/src/component.rs`, add after the existing modules:

```rust
pub mod v5 {
    wasmtime::component::bindgen!({
        path: "./wit/v5",
        world: "bridge",
    });
}
```

#### 3. Generate type conversions

Add the conversion macro call:

```rust
impl_conversions!(v5);
```

#### 4. Add BridgeWrapper variant

Update the enum:

```rust
pub enum BridgeWrapper {
    V3(v3::Bridge),
    V4(v4::Bridge),
    V5(v5::Bridge),  // <-- add this
}
```

Add an arm to each `bridge_method!` macro expansion. In the macro definitions, add:

```rust
BridgeWrapper::V5(b) => {
    let r = b.$interface().$method(store, ...)?;
    Ok(r.map(Into::into).map_err(Into::into))
}
```

#### 5. Register interfaces in the linker

In `build_linker()`:

```rust
// v5: http + environment + file + new_feature
v5::standout::app::http::add_to_linker(&mut linker, |s| s)?;
v5::standout::app::environment::add_to_linker(&mut linker, |s| s)?;
v5::standout::app::file::add_to_linker(&mut linker, |s| s)?;
v5::standout::app::new_feature::add_to_linker(&mut linker, |s| s)?;  // if applicable
```

#### 6. Update the instantiation chain

In `app()`, add v5 at the top (newest first):

```rust
// v5 (newest)
if let Ok(instance) = v5::Bridge::instantiate(&mut *store, &component, &linker) {
    return Ok(BridgeWrapper::V5(instance));
}

// v4
if let Ok(instance) = v4::Bridge::instantiate(&mut *store, &component, &linker) {
    return Ok(BridgeWrapper::V4(instance));
}

// v3 (oldest)
// ...
```

#### 7. Register Host implementations

In `app_state.rs`:

```rust
impl_host_for_version!(v5);
```

In `request_builder.rs`:

```rust
impl_host_request_builder!(v5);
impl_http_type_conversions!(v5);
```

#### 8. If the version has the file interface

In `file_ops.rs` (only if v5 includes the `file` interface):

```rust
impl_file_host!(v5);
```

#### 9. If adding new host functions

If the new version introduces entirely new interfaces (not just `file`), implement the `Host` trait:

```rust
impl v5::standout::app::new_feature::Host for AppState {
    fn some_function(&mut self, ...) -> ... {
        // implementation
    }
}
```

### Benefits

- **No forced rebuilds**: Existing connectors continue to work after gem updates
- **Gradual migration**: Update connectors to new WIT versions at your own pace
- **Type safety**: Each version's types are converted to the latest version internally

## Development

To contribute or modify this gem, ensure you have the following dependencies installed:

- **Ruby 3.3.0** (or later)
- **Rust 1.84.0** (or later)

### Setting Up the Development Environment

Run the following command to setup and install additional dependencies:

```bash
bin/setup
```

Then, to compile example applications, run tests, and perform syntax checks, execute:

```bash
rake
```

### Useful Commands

- **Interactive Console:** Run `bin/console` to interactively test the code.
- **Full Test Suite & Linting:** Run `rake` to compile, execute tests, and perform syntax checks.
- **Run Tests Only:** Execute `rake spec` to run only the test suite.
- **Linting:** Run `rake rubocop` to check code style and formatting.
- **Compile Example Applications:** Use `rake fixtures` to build the example apps.

To install this gem locally for testing purposes, run:

```bash
bundle exec rake install
```

## Release & Distribution

To release a new version of the gem, update the version number in `lib/app_bridge/version.rb` and in `ext/app_bridge/Cargo.toml`. They should be the same.

Then push the changes to the repository and create a new release on GitHub. The gem will be automatically built and published to RubyGems.
