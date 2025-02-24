# Steps to Add WebAssembly Support

## Prerequisites

1. **Install `wasm-pack`:**
   `wasm-pack` is a tool that helps you build and publish Rust-generated WebAssembly. Install it using Cargo:
   ```sh
   cargo install wasm-pack
   ```

## Update `Cargo.toml`

2. **Add Dependencies:**
   Update your `Cargo.toml` to include the necessary dependencies for WebAssembly:
   ```toml
   [dependencies]
   wasm-bindgen = "0.2"
   web-sys = { version = "0.3", features = ["console"] }
   ```

## Create `lib.rs`

3. **Create a `lib.rs` File:**
   Create a `lib.rs` file in the `src` directory with the following content:
   ```rust
   use wasm_bindgen::prelude::*;

   #[wasm_bindgen]
   pub fn greet(name: &str) {
       web_sys::console::log_1(&format!("Hello, {}!", name).into());
   }
   ```

## Configure `wasm-pack`

4. **Create a `wasm-pack.toml` File:**
   Create a `wasm-pack.toml` file in your project root with the following content:
   ```toml
   [package]
   name = "mcp_daemon"
   version = "0.2.1"
   authors = ["Shawn McAllister"]
   description = "Diverged Implementation of Model Context Protocol (MCP) with Extended Functionality"
   license = "Apache-2.0"
   repository = "https://github.com/entrepeneur4lyf/mcp-daemon"
   ```

## Build the WebAssembly Module

5. **Build the WebAssembly Module:**
   Use `wasm-pack` to build the WebAssembly module:
   ```sh
   wasm-pack build --target web
   ```

## Create an HTML File

6. **Create an `index.html` File:**
   Create an `index.html` file in your project root with the following content:
   ```html
   <!DOCTYPE html>
   <html lang="en">
   <head>
       <meta charset="UTF-8">
       <meta name="viewport" content="width=device-width, initial-scale=1.0">
       <title>MCP Daemon</title>
   </head>
   <body>
       <script type="module">
           import init, { greet } from './pkg/mcp_daemon.js';

           async function run() {
               await init();
               greet("World");
           }

           run();
       </script>
   </body>
   </html>
   ```

## Serve the HTML File

7. **Serve the HTML File:**
   Serve the `index.html` file using a simple HTTP server. You can use `python` to serve the file:
   ```sh
   python -m http.server 8000
   ```

## Open the HTML File in a Browser

8. **Open the HTML File in a Browser:**
   Open the `index.html` file in a web browser to see the WebAssembly module in action.

## Example Commands

```sh
# Install wasm-pack
cargo install wasm-pack

# Update Cargo.toml
echo '
[dependencies]
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["console"] }
' >> Cargo.toml

# Create lib.rs
echo '
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: &str) {
    web_sys::console::log_1(&format!("Hello, {}!", name).into());
}
' > src/lib.rs

# Create wasm-pack.toml
echo '
[package]
name = "mcp_daemon"
version = "0.2.1"
authors = ["Shawn McAllister"]
description = "Diverged Implementation of Model Context Protocol (MCP) with Extended Functionality"
license = "Apache-2.0"
repository = "https://github.com/entrepeneur4lyf/mcp-daemon"
' > wasm-pack.toml

# Build the WebAssembly module
wasm-pack build --target web

# Create index.html
echo '
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MCP Daemon</title>
</head>
<body>
    <script type="module">
        import init, { greet } from "./pkg/mcp_daemon.js";

        async function run() {
            await init();
            greet("World");
        }

        run();
    </script>
</body>
</html>
' > index.html

# Serve the HTML file
python -m http.server 8000