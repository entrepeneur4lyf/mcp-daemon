# Steps to Create Cross-Platform Binaries for npm Package

## Prerequisites

1. **Install Rust and Cargo:**
   Ensure you have Rust and Cargo installed on your system. You can install them using `rustup`:
   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install `cross`:**
   `cross` is a tool that helps you compile Rust projects for different platforms. Install it using Cargo:
   ```sh
   cargo install cross
   ```

## Configuration

3. **Configure `cross`:**
   Create a `.cargo/config.toml` file in your project root with the following content to configure `cross`:
   ```toml
   [target.x86_64-pc-windows-gnu]
   linker = "x86_64-w64-mingw32-gcc"

   [target.x86_64-apple-darwin]
   linker = "x86_64-apple-darwin14-clang"

   [target.aarch64-apple-darwin]
   linker = "aarch64-apple-darwin20-clang"

   [target.x86_64-unknown-linux-gnu]
   linker = "x86_64-unknown-linux-gnu-gcc"

   [target.aarch64-unknown-linux-gnu]
   linker = "aarch64-unknown-linux-gnu-gcc"
   ```

## Build Cross-Platform Binaries

4. **Build Cross-Platform Binaries:**
   Use `cross` to build binaries for different platforms. For example, to build for Windows, macOS, and Linux:
   ```sh
   cross build --target x86_64-pc-windows-gnu --release
   cross build --target x86_64-apple-darwin --release
   cross build --target aarch64-apple-darwin --release
   cross build --target x86_64-unknown-linux-gnu --release
   cross build --target aarch64-unknown-linux-gnu --release
   ```

## Package the Binaries

5. **Package the Binaries:**
   Create a directory structure for your npm package and copy the binaries into it. For example:
   ```sh
   mkdir -p npm-package/bin
   cp target/x86_64-pc-windows-gnu/release/mcp_daemon npm-package/bin/mcp_daemon.exe
   cp target/x86_64-apple-darwin/release/mcp_daemon npm-package/bin/mcp_daemon-macos
   cp target/aarch64-apple-darwin/release/mcp_daemon npm-package/bin/mcp_daemon-macos-arm64
   cp target/x86_64-unknown-linux-gnu/release/mcp_daemon npm-package/bin/mcp_daemon-linux
   cp target/aarch64-unknown-linux-gnu/release/mcp_daemon npm-package/bin/mcp_daemon-linux-arm64
   ```

## Create `package.json`

6. **Create `package.json`:**
   Create a `package.json` file in the `npm-package` directory with the following content:
   ```json
   {
     "name": "mcp-daemon",
     "version": "0.2.1",
     "description": "Diverged Implementation of Model Context Protocol (MCP) with Extended Functionality",
     "main": "index.js",
     "bin": {
       "mcp-daemon": "./bin/mcp_daemon"
     },
     "scripts": {
       "start": "node index.js"
     },
     "keywords": [
       "async",
       "mcp",
       "protocol",
       "daemon",
       "extended"
     ],
     "author": "Shawn McAllister",
     "license": "Apache-2.0",
     "os": [
       "win32",
       "darwin",
       "linux"
     ],
     "cpu": [
       "x64",
       "arm64"
     ]
   }
   ```

## Create `index.js`

7. **Create `index.js`:**
   Create an `index.js` file in the `npm-package` directory to handle the binary execution:
   ```js
   const { execFile } = require('child_process');
   const path = require('path');

   const binPath = path.join(__dirname, 'bin', 'mcp_daemon');

   execFile(binPath, (error, stdout, stderr) => {
     if (error) {
       console.error(`Error: ${error.message}`);
       return;
     }
     if (stderr) {
       console.error(`Stderr: ${stderr}`);
       return;
     }
     console.log(`Stdout: ${stdout}`);
   });
   ```

## Publish the npm Package

8. **Publish the npm Package:**
   Publish the npm package to the npm registry:
   ```sh
   cd npm-package
   npm publish
   ```

## Example Commands

```sh
# Install Rust and Cargo
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install cross
cargo install cross

# Configure cross
echo "
[target.x86_64-pc-windows-gnu]
linker = \"x86_64-w64-mingw32-gcc\"

[target.x86_64-apple-darwin]
linker = \"x86_64-apple-darwin14-clang\"

[target.aarch64-apple-darwin]
linker = \"aarch64-apple-darwin20-clang\"

[target.x86_64-unknown-linux-gnu]
linker = \"x86_64-unknown-linux-gnu-gcc\"

[target.aarch64-unknown-linux-gnu]
linker = \"aarch64-unknown-linux-gnu-gcc\"
" > .cargo/config.toml

# Build cross-platform binaries
cross build --target x86_64-pc-windows-gnu --release
cross build --target x86_64-apple-darwin --release
cross build --target aarch64-apple-darwin --release
cross build --target x86_64-unknown-linux-gnu --release
cross build --target aarch64-unknown-linux-gnu --release

# Package the binaries
mkdir -p npm-package/bin
cp target/x86_64-pc-windows-gnu/release/mcp_daemon npm-package/bin/mcp_daemon.exe
cp target/x86_64-apple-darwin/release/mcp_daemon npm-package/bin/mcp_daemon-macos
cp target/aarch64-apple-darwin/release/mcp_daemon npm-package/bin/mcp_daemon-macos-arm64
cp target/x86_64-unknown-linux-gnu/release/mcp_daemon npm-package/bin/mcp_daemon-linux
cp target/aarch64-unknown-linux-gnu/release/mcp_daemon npm-package/bin/mcp_daemon-linux-arm64

# Create package.json
echo '
{
  "name": "mcp-daemon",
  "version": "0.2.1",
  "description": "Diverged Implementation of Model Context Protocol (MCP) with Extended Functionality",
  "main": "index.js",
  "bin": {
    "mcp-daemon": "./bin/mcp_daemon"
  },
  "scripts": {
    "start": "node index.js"
  },
  "keywords": [
    "async",
    "mcp",
    "protocol",
    "daemon",
    "extended"
  ],
  "author": "Shawn McAllister",
  "license": "Apache-2.0",
  "os": [
    "win32",
    "darwin",
    "linux"
  ],
  "cpu": [
    "x64",
    "arm64"
  ]
}
' > npm-package/package.json

# Create index.js
echo '
const { execFile } = require("child_process");
const path = require("path");

const binPath = path.join(__dirname, "bin", "mcp_daemon");

execFile(binPath, (error, stdout, stderr) => {
  if (error) {
    console.error(`Error: ${error.message}`);
    return;
  }
  if (stderr) {
    console.error(`Stderr: ${stderr}`);
    return;
  }
  console.log(`Stdout: ${stdout}`);
});
' > npm-package/index.js

# Publish the npm package
cd npm-package
npm publish