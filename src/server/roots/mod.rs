//! # MCP Roots System
//!
//! This module provides types and utilities for implementing MCP's roots system,
//! which defines boundaries where servers can operate.
//!
//! ## Overview
//!
//! Roots in MCP define the boundaries where servers can operate, typically
//! representing filesystem paths or API endpoints. They serve several purposes:
//!
//! * Provide guidance to servers about relevant resources
//! * Define security boundaries for resource access
//! * Organize workspaces and resources
//! * Enable validation of resource URIs
//!
//! Clients can declare roots to servers, and servers can check if resources
//! fall within these declared roots.
//!
//! ## Examples
//!
//! ```rust
//! use mcp_daemon::server::roots::{Root, RootExt, RegisteredRoots};
//! use url::Url;
//!
//! // Define roots for a workspace
//! let roots = vec![
//!     Root {
//!         uri: "file:///home/user/projects".to_string(),
//!         name: Some("Projects".to_string()),
//!     },
//!     Root {
//!         uri: "https://api.example.com".to_string(),
//!         name: Some("API Endpoint".to_string()),
//!     },
//! ];
//!
//! // Check if a resource is within the roots
//! let resource_url = Url::parse("file:///home/user/projects/app/src/main.rs").unwrap();
//! assert!(resource_url.is_within_roots(&roots));
//!
//! // Create a roots handler
//! let handler = RegisteredRoots::new(
//!     || {
//!         Box::pin(async {
//!             Ok(vec![
//!                 Root {
//!                     uri: "file:///home/user/projects".to_string(),
//!                     name: Some("Projects".to_string()),
//!                 },
//!             ])
//!         })
//!     },
//!     true, // supports change notifications
//! );
//! ```
//!
//! ## Related Modules
//!
//! * [`crate::server`] - Base server implementation
//! * [`crate::server::resource`] - Resource system that uses roots for validation

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use url::Url;

/// A root that defines a boundary where a server can operate
///
/// Roots typically represent filesystem paths or API endpoints and
/// define the boundaries where servers can operate.
///
/// # Fields
///
/// * `uri` - The URI of the root
/// * `name` - Optional human-readable name for the root
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::roots::Root;
///
/// // Create a filesystem root
/// let file_root = Root {
///     uri: "file:///home/user/projects".to_string(),
///     name: Some("Projects".to_string()),
/// };
///
/// // Create an API endpoint root
/// let api_root = Root {
///     uri: "https://api.example.com".to_string(),
///     name: Some("API Endpoint".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Root {
    /// The URI of the root
    pub uri: String,
    /// Optional human-readable name for the root
    pub name: Option<String>,
}

/// A callback that can list roots
///
/// This trait defines the interface for components that can provide
/// a list of roots.
///
/// # Examples
///
/// ```rust
/// use std::pin::Pin;
/// use std::future::Future;
/// use mcp_daemon::server::roots::{RootsCallback, Root};
///
/// struct MyRootsCallback;
///
/// impl RootsCallback for MyRootsCallback {
///     fn call(&self) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<Root>>> + Send>> {
///         Box::pin(async {
///             Ok(vec![
///                 Root {
///                     uri: "file:///home/user/projects".to_string(),
///                     name: Some("Projects".to_string()),
///                 },
///             ])
///         })
///     }
/// }
/// ```
pub trait RootsCallback: Send + Sync {
    /// Calls the roots function to get a list of root directories
    ///
    /// # Returns
    ///
    /// A future that resolves to a list of roots
    fn call(&self) -> RootsFuture;
}

// Type aliases for complex future and callback types
type RootsFuture = Pin<Box<dyn Future<Output = anyhow::Result<Vec<Root>>> + Send>>;
type RootsCallbackFunc = Box<dyn Fn() -> RootsFuture + Send + Sync>;

struct RootsCallbackFn(RootsCallbackFunc);

impl RootsCallback for RootsCallbackFn {
    fn call(&self) -> RootsFuture {
        (self.0)()
    }
}

/// A registered roots handler
///
/// Represents a roots handler that has been registered with the server,
/// including its callback and change notification support.
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::roots::{RegisteredRoots, Root};
///
/// // Create a roots handler
/// let handler = RegisteredRoots::new(
///     || {
///         Box::pin(async {
///             Ok(vec![
///                 Root {
///                     uri: "file:///home/user/projects".to_string(),
///                     name: Some("Projects".to_string()),
///                 },
///             ])
///         })
///     },
///     true, // supports change notifications
/// );
/// ```
pub(crate) struct RegisteredRoots {
    /// The callback to list roots
    #[allow(dead_code)]
    pub callback: Arc<dyn RootsCallback>,
    /// Whether the handler supports root change notifications
    #[allow(dead_code)]
    pub supports_change_notifications: bool,
}

impl RegisteredRoots {
    /// Create a new roots handler with the given callback
    ///
    /// # Arguments
    ///
    /// * `list_callback` - The callback to list roots
    /// * `supports_change_notifications` - Whether the handler supports root change notifications
    ///
    /// # Returns
    ///
    /// A new RegisteredRoots instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::roots::{RegisteredRoots, Root};
    ///
    /// let handler = RegisteredRoots::new(
    ///     || {
    ///         Box::pin(async {
    ///             Ok(vec![
    ///                 Root {
    ///                     uri: "file:///home/user/projects".to_string(),
    ///                     name: Some("Projects".to_string()),
    ///                 },
    ///             ])
    ///         })
    ///     },
    ///     true, // supports change notifications
    /// );
    /// ```
    pub fn new(
        list_callback: impl Fn() -> RootsFuture + Send + Sync + 'static,
        supports_change_notifications: bool,
    ) -> Self {
        Self {
            callback: Arc::new(RootsCallbackFn(Box::new(list_callback))),
            supports_change_notifications,
        }
    }

    /// List all available roots
    ///
    /// # Returns
    ///
    /// A future that resolves to a list of roots
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::roots::{RegisteredRoots, Root};
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let handler = RegisteredRoots::new(
    ///     || {
    ///         Box::pin(async {
    ///             Ok(vec![
    ///                 Root {
    ///                     uri: "file:///home/user/projects".to_string(),
    ///                     name: Some("Projects".to_string()),
    ///                 },
    ///             ])
    ///         })
    ///     },
    ///     true,
    /// );
    ///
    /// let roots = handler.list_roots().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[allow(dead_code)]
    pub async fn list_roots(&self) -> anyhow::Result<Vec<Root>> {
        self.callback.call().await
    }
}

/// Extension trait for working with roots
///
/// Provides utility methods for checking if URLs are within roots.
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::roots::{Root, RootExt};
/// use url::Url;
///
/// let roots = vec![
///     Root {
///         uri: "file:///home/user/projects".to_string(),
///         name: None,
///     },
///     Root {
///         uri: "https://api.example.com".to_string(),
///         name: None,
///     },
/// ];
///
/// let url = Url::parse("file:///home/user/projects/app/src/main.rs").unwrap();
/// assert!(url.is_within_roots(&roots));
///
/// let url = Url::parse("https://other.com/api").unwrap();
/// assert!(!url.is_within_roots(&roots));
/// ```
pub trait RootExt {
    /// Check if a URI is within any of the given roots
    ///
    /// # Arguments
    ///
    /// * `roots` - The roots to check against
    ///
    /// # Returns
    ///
    /// `true` if the URI is within any of the roots, `false` otherwise
    fn is_within_roots(&self, roots: &[Root]) -> bool;
}

impl RootExt for Url {
    fn is_within_roots(&self, roots: &[Root]) -> bool {
        roots.iter().any(|root| {
            if let Ok(root_url) = Url::parse(&root.uri) {
                self.as_str().starts_with(root_url.as_str())
            } else {
                false
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_roots_handler() {
        let handler = RegisteredRoots::new(
            || {
                Box::pin(async {
                    Ok(vec![
                        Root {
                            uri: "file:///home/user/projects".to_string(),
                            name: Some("Projects".to_string()),
                        },
                        Root {
                            uri: "https://api.example.com".to_string(),
                            name: None,
                        },
                    ])
                })
            },
            true,
        );

        let roots = handler.list_roots().await.unwrap();
        assert_eq!(roots.len(), 2);
        assert_eq!(roots[0].name, Some("Projects".to_string()));
        assert_eq!(roots[1].uri, "https://api.example.com");
    }

    #[test]
    fn test_url_within_roots() {
        let roots = vec![
            Root {
                uri: "file:///home/user/projects".to_string(),
                name: None,
            },
            Root {
                uri: "https://api.example.com".to_string(),
                name: None,
            },
        ];

        let url1 = Url::parse("file:///home/user/projects/app/src/main.rs").unwrap();
        let url2 = Url::parse("https://api.example.com/v1/users").unwrap();
        let url3 = Url::parse("https://other.com/api").unwrap();

        assert!(url1.is_within_roots(&roots));
        assert!(url2.is_within_roots(&roots));
        assert!(!url3.is_within_roots(&roots));
    }
}
