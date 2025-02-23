use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use url::Url;

/// A root that defines a boundary where a server can operate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Root {
    /// The URI of the root
    pub uri: String,
    /// Optional human-readable name for the root
    pub name: Option<String>,
}

/// A callback that can list roots
pub trait RootsCallback: Send + Sync {
    /// Calls the roots function to get a list of root directories
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
    #[allow(dead_code)]
    pub async fn list_roots(&self) -> anyhow::Result<Vec<Root>> {
        self.callback.call().await
    }
}

/// Extension trait for working with roots
pub trait RootExt {
    /// Check if a URI is within any of the given roots
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
