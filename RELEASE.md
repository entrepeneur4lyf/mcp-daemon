# Steps to Create a Release and Publish

## Prepare the Project for Release

1. **Ensure all code is committed and pushed to the repository.**
2. **Update the `Cargo.toml` file with the new version number.**
   - Example:
     ```toml
     [package]
     name = "mcp_daemon"
     version = "0.2.1"
     edition = "2021"
     ```

3. **Commit and push the changes to the repository.**
   ```sh
   git add Cargo.toml
   git commit -m "Prepare for release 0.2.1"
   git push origin main
   ```

## Build the Release

4. **Build the project in release mode.**
   ```sh
   cargo build --release
   ```

## Test the Release

5. **Run tests to ensure the release build works as expected.**
   ```sh
   cargo test --release
   ```

## Publish the Release

6. **Publish the crate to crates.io.**
   ```sh
   cargo publish
   ```

## Create a GitHub Release

7. **Go to the GitHub repository.**
8. **Click on "Releases" and then "Draft a new release".**
9. **Tag the release with the version number (e.g., `v0.2.1`).**
10. **Write release notes describing the changes.**
11. **Attach the binary artifacts if necessary.**
12. **Publish the release.**

## Additional Information

- **Profiles:** The Cargo Book provides detailed information on profiles and their settings. You can refer to the [Profiles section](https://doc.rust-lang.org/cargo/reference/profiles.html#default-profiles) for more details.
- **Publishing:** Ensure you have the necessary permissions to publish the crate to crates.io. You may need to log in using `cargo login <api-token>`.

## Example Commands

```sh
# Update Cargo.toml version
# Commit and push changes
git add Cargo.toml
git commit -m "Prepare for release 0.2.1"
git push origin main

# Build the release
cargo build --release

# Run tests
cargo test --release

# Publish the crate
cargo publish

# Create a GitHub release
# Go to GitHub repository
# Click on "Releases" -> "Draft a new release"
# Tag the release with the version number (e.g., v0.2.1)
# Write release notes
# Attach binary artifacts if necessary
# Publish the release