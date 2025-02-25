//! # MCP Resource System
//!
//! This module provides types and utilities for implementing MCP's resource system,
//! which allows servers to expose data and content to clients.
//!
//! ## Overview
//!
//! The resource system enables servers to expose various types of data:
//!
//! * Static resources with fixed URIs
//! * Dynamic resources through URI templates
//! * Subscribable resources that notify clients of updates
//! * Resources with completion support for URI variables
//!
//! Resources can contain text or binary data and are identified by unique URIs.
//!
//! ## Examples
//!
//! ```rust
//! use std::collections::HashMap;
//! use url::Url;
//! use mcp_daemon::server::resource::{ResourceTemplate, RegisteredResource};
//! use mcp_daemon::types::{Resource, ResourceContents};
//! use mcp_daemon::completable::CompletableString;
//!
//! // Create a static resource
//! let resource_uri = Url::parse("file:///example.txt").unwrap();
//! let resource = Resource {
//!     uri: resource_uri.clone(),
//!     name: "Example File".to_string(),
//!     description: Some("An example text file".to_string()),
//!     mime_type: Some("text/plain".to_string()),
//! };
//!
//! // Register the resource with a read callback
//! let registered = RegisteredResource::new(
//!     resource,
//!     |uri: &Url| async move {
//!         vec![ResourceContents {
//!             uri: uri.clone(),
//!             text: Some("This is the content of the example file.".to_string()),
//!             blob: None,
//!             mime_type: Some("text/plain".to_string()),
//!         }]
//!     },
//!     true, // supports subscriptions
//! );
//!
//! // Create a resource template with variable completion
//! let template = ResourceTemplate::new("file:///{path}")
//!     .with_list(|| async {
//!         // Return a list of resources matching this template
//!         vec![
//!             Resource {
//!                 uri: Url::parse("file:///example1.txt").unwrap(),
//!                 name: "Example 1".to_string(),
//!                 description: None,
//!                 mime_type: Some("text/plain".to_string()),
//!             },
//!             Resource {
//!                 uri: Url::parse("file:///example2.txt").unwrap(),
//!                 name: "Example 2".to_string(),
//!                 description: None,
//!                 mime_type: Some("text/plain".to_string()),
//!             },
//!         ]
//!     })
//!     .with_completion(
//!         "path",
//!         CompletableString::new(|input: &str| {
//!             let input = input.to_string();
//!             async move {
//!                 vec!["example1.txt", "example2.txt"]
//!                     .into_iter()
//!                     .filter(|path| path.starts_with(&input))
//!                     .map(|path| path.to_string())
//!                     .collect()
//!             }
//!         }),
//!     );
//! ```
//!
//! ## Related Modules
//!
//! * [`crate::server`] - Base server implementation
//! * [`crate::completable`] - Completion system for resource URI variables
//! * [`crate::types`] - Core MCP types including Resource and ResourceContents

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use url::Url;

use crate::completable::Completable;
use crate::types::{Resource, ResourceContents};
use std::collections::HashSet;
use tokio::sync::broadcast;

/// Result type for listing resources
///
/// A vector of Resource objects returned by list operations.
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::resource::ListResourcesResult;
/// use mcp_daemon::types::Resource;
/// use url::Url;
///
/// let resources: ListResourcesResult = vec![
///     Resource {
///         uri: Url::parse("file:///example.txt").unwrap(),
///         name: "Example File".to_string(),
///         description: None,
///         mime_type: Some("text/plain".to_string()),
///     }
/// ];
/// ```
pub type ListResourcesResult = Vec<Resource>;

/// Result type for reading resources
///
/// A vector of ResourceContents objects returned by read operations.
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::resource::ReadResourceResult;
/// use mcp_daemon::types::ResourceContents;
/// use url::Url;
///
/// let contents: ReadResourceResult = vec![
///     ResourceContents {
///         uri: Url::parse("file:///example.txt").unwrap(),
///         text: Some("File content".to_string()),
///         blob: None,
///         mime_type: Some("text/plain".to_string()),
///     }
/// ];
/// ```
pub type ReadResourceResult = Vec<ResourceContents>;

// Type aliases for complex Future types
type ListResourcesFuture = Pin<Box<dyn Future<Output = ListResourcesResult> + Send + 'static>>;
type ReadResourceFuture = Pin<Box<dyn Future<Output = ReadResourceResult> + Send + 'static>>;

// Type aliases for callback functions
type ListResourcesFn = Box<dyn Fn() -> ListResourcesFuture + Send + Sync>;
type ReadResourceFn = Box<dyn Fn(&Url) -> ReadResourceFuture + Send + Sync>;
type ArcReadResourceCallback = Arc<dyn ReadResourceCallback>;



/// A channel for resource update notifications
///
/// Provides a mechanism for notifying clients when resources are updated.
/// Clients can subscribe to updates for specific resources and receive
/// notifications when those resources change.
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::resource::ResourceUpdateChannel;
/// use url::Url;
///
/// // Create a new update channel
/// let channel = ResourceUpdateChannel::new();
///
/// // Subscribe to updates for a resource
/// let uri = Url::parse("file:///example.txt").unwrap();
/// let mut receiver = channel.subscribe(&uri);
///
/// // Notify subscribers of an update
/// channel.notify_update(&uri);
///
/// // Unsubscribe from updates
/// channel.unsubscribe(&uri);
/// ```
#[derive(Clone)]
pub struct ResourceUpdateChannel {
    /// The sender for resource updates
    sender: broadcast::Sender<String>,
    /// The set of subscribed URIs
    subscribed_uris: Arc<RwLock<HashSet<String>>>,
}

impl ResourceUpdateChannel {
    /// Create a new resource update channel
    ///
    /// # Returns
    ///
    /// A new ResourceUpdateChannel instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::resource::ResourceUpdateChannel;
    ///
    /// let channel = ResourceUpdateChannel::new();
    /// ```
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self {
            sender,
            subscribed_uris: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Subscribe to updates for a resource
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI of the resource to subscribe to
    ///
    /// # Returns
    ///
    /// A broadcast receiver for update notifications
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::resource::ResourceUpdateChannel;
    /// use url::Url;
    ///
    /// let channel = ResourceUpdateChannel::new();
    /// let uri = Url::parse("file:///example.txt").unwrap();
    /// let receiver = channel.subscribe(&uri);
    /// ```
    pub fn subscribe(&self, uri: &Url) -> broadcast::Receiver<String> {
        self.subscribed_uris.write().unwrap().insert(uri.to_string());
        self.sender.subscribe()
    }

    /// Unsubscribe from updates for a resource
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI of the resource to unsubscribe from
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::resource::ResourceUpdateChannel;
    /// use url::Url;
    ///
    /// let channel = ResourceUpdateChannel::new();
    /// let uri = Url::parse("file:///example.txt").unwrap();
    /// let receiver = channel.subscribe(&uri);
    /// channel.unsubscribe(&uri);
    /// ```
    pub fn unsubscribe(&self, uri: &Url) {
        self.subscribed_uris.write().unwrap().remove(&uri.to_string());
    }

    /// Send an update notification for a resource
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI of the resource that was updated
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::resource::ResourceUpdateChannel;
    /// use url::Url;
    ///
    /// let channel = ResourceUpdateChannel::new();
    /// let uri = Url::parse("file:///example.txt").unwrap();
    /// let receiver = channel.subscribe(&uri);
    /// channel.notify_update(&uri);
    /// ```
    pub fn notify_update(&self, uri: &Url) {
        if self.subscribed_uris.read().unwrap().contains(&uri.to_string()) {
            let _ = self.sender.send(uri.to_string());
        }
    }
}

impl Default for ResourceUpdateChannel {
    fn default() -> Self {
        Self::new()
    }
}

/// A template for resources that can be dynamically generated
///
/// Resource templates allow servers to define patterns for resources
/// that can be dynamically generated based on URI variables. They can
/// also provide completion for URI variables and list all resources
/// matching the template.
///
/// # Examples
///
/// ```rust
/// use mcp_daemon::server::resource::ResourceTemplate;
/// use mcp_daemon::completable::CompletableString;
///
/// // Create a template for file resources
/// let template = ResourceTemplate::new("file:///{path}")
///     .with_list(|| async {
///         // Return a list of resources matching this template
///         vec![]
///     })
///     .with_completion(
///         "path",
///         CompletableString::new(|input: &str| {
///             let input = input.to_string();
///             async move {
///                 vec!["example.txt", "readme.md"]
///                     .into_iter()
///                     .filter(|path| path.starts_with(&input))
///                     .map(|path| path.to_string())
///                     .collect()
///             }
///         }),
///     );
/// ```
pub struct ResourceTemplate {
    /// The URI template pattern
    uri_template: String,
    /// Optional callback to list all resources matching this template
    list_callback: Option<Arc<dyn ListResourcesCallback>>,
    /// Optional callbacks to complete template variables
    complete_callbacks: HashMap<String, Arc<dyn Completable<Input = str, Output = String>>>,
}

impl ResourceTemplate {
    /// Create a new resource template with the given URI pattern
    ///
    /// # Arguments
    ///
    /// * `uri_template` - The URI template pattern (e.g., "file:///{path}")
    ///
    /// # Returns
    ///
    /// A new ResourceTemplate instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::resource::ResourceTemplate;
    ///
    /// let template = ResourceTemplate::new("file:///{path}");
    /// ```
    pub fn new(uri_template: impl Into<String>) -> Self {
        Self {
            uri_template: uri_template.into(),
            list_callback: None,
            complete_callbacks: HashMap::new(),
        }
    }

    /// Add a callback to list all resources matching this template
    ///
    /// # Arguments
    ///
    /// * `callback` - A function that returns a future resolving to a list of resources
    ///
    /// # Returns
    ///
    /// The updated ResourceTemplate
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::resource::ResourceTemplate;
    /// use mcp_daemon::types::Resource;
    /// use url::Url;
    ///
    /// let template = ResourceTemplate::new("file:///{path}")
    ///     .with_list(|| async {
    ///         vec![
    ///             Resource {
    ///                 uri: Url::parse("file:///example.txt").unwrap(),
    ///                 name: "Example File".to_string(),
    ///                 description: None,
    ///                 mime_type: Some("text/plain".to_string()),
    ///             }
    ///         ]
    ///     });
    /// ```
    pub fn with_list<F, Fut>(mut self, callback: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ListResourcesResult> + Send + 'static,
    {
        self.list_callback = Some(Arc::new(ListResourcesCallbackFn(Box::new(move || {
            Box::pin(callback())
        }))));
        self
    }

    /// Add a completion callback for a template variable
    ///
    /// # Arguments
    ///
    /// * `variable` - The name of the template variable
    /// * `completable` - The completion implementation
    ///
    /// # Returns
    ///
    /// The updated ResourceTemplate
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::resource::ResourceTemplate;
    /// use mcp_daemon::completable::CompletableString;
    ///
    /// let template = ResourceTemplate::new("file:///{path}")
    ///     .with_completion(
    ///         "path",
    ///         CompletableString::new(|input: &str| {
    ///             let input = input.to_string();
    ///             async move {
    ///                 vec!["example.txt", "readme.md"]
    ///                     .into_iter()
    ///                     .filter(|path| path.starts_with(&input))
    ///                     .map(|path| path.to_string())
    ///                     .collect()
    ///             }
    ///         }),
    ///     );
    /// ```
    pub fn with_completion(
        mut self,
        variable: impl Into<String>,
        completable: impl Completable<Input = str, Output = String> + 'static,
    ) -> Self {
        self.complete_callbacks
            .insert(variable.into(), Arc::new(completable));
        self
    }

    /// Get the URI template pattern
    ///
    /// # Returns
    ///
    /// The URI template pattern string
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::resource::ResourceTemplate;
    ///
    /// let template = ResourceTemplate::new("file:///{path}");
    /// assert_eq!(template.uri_template(), "file:///{path}");
    /// ```
    pub fn uri_template(&self) -> &str {
        &self.uri_template
    }

    /// Get the list callback if one exists
    ///
    /// # Returns
    ///
    /// An optional reference to the list callback
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::resource::ResourceTemplate;
    ///
    /// let template = ResourceTemplate::new("file:///{path}")
    ///     .with_list(|| async { vec![] });
    ///
    /// assert!(template.list_callback().is_some());
    /// ```
    pub fn list_callback(&self) -> Option<&dyn ListResourcesCallback> {
        self.list_callback.as_deref()
    }

    /// Get the completion callback for a variable if one exists
    ///
    /// # Arguments
    ///
    /// * `variable` - The name of the template variable
    ///
    /// # Returns
    ///
    /// An optional reference to the completion callback
    ///
    /// # Examples
    ///
    /// ```rust
    /// use mcp_daemon::server::resource::ResourceTemplate;
    /// use mcp_daemon::completable::CompletableString;
    ///
    /// let template = ResourceTemplate::new("file:///{path}")
    ///     .with_completion(
    ///         "path",
    ///         CompletableString::new(|input: &str| {
    ///             let input = input.to_string();
    ///             async move { vec![format!("{}/file.txt", input)] }
    ///         }),
    ///     );
    ///
    /// assert!(template.complete_callback("path").is_some());
    /// assert!(template.complete_callback("nonexistent").is_none());
    /// ```
    pub fn complete_callback(
        &self,
        variable: &str,
    ) -> Option<&dyn Completable<Input = str, Output = String>> {
        self.complete_callbacks.get(variable).map(|c| c.as_ref())
    }
}

/// A callback that can list resources matching a template
///
/// This trait defines the interface for components that can list
/// resources matching a template.
///
/// # Examples
///
/// ```rust
/// use std::pin::Pin;
/// use std::future::Future;
/// use mcp_daemon::server::resource::{ListResourcesCallback, ListResourcesResult};
///
/// struct MyListResourcesCallback;
///
/// impl ListResourcesCallback for MyListResourcesCallback {
///     fn call(&self) -> Pin<Box<dyn Future<Output = ListResourcesResult> + Send + 'static>> {
///         Box::pin(async { vec![] })
///     }
/// }
/// ```
pub trait ListResourcesCallback: Send + Sync {
    /// Calls the list resources function
    ///
    /// # Returns
    ///
    /// A future that resolves to a list of resources
    fn call(&self) -> ListResourcesFuture;
}

/// Callback function for listing resources
///
/// Wraps a function that returns a future resolving to a list of resources.
pub struct ListResourcesCallbackFn(ListResourcesFn);

impl ListResourcesCallback for ListResourcesCallbackFn {
    fn call(&self) -> ListResourcesFuture {
        (self.0)()
    }
}

/// A callback that can read a resource
///
/// This trait defines the interface for components that can read
/// resources by URI.
///
/// # Examples
///
/// ```rust
/// use std::pin::Pin;
/// use std::future::Future;
/// use url::Url;
/// use mcp_daemon::server::resource::{ReadResourceCallback, ReadResourceResult};
///
/// struct MyReadResourceCallback;
///
/// impl ReadResourceCallback for MyReadResourceCallback {
///     fn call(&self, uri: &Url) -> Pin<Box<dyn Future<Output = ReadResourceResult> + Send + 'static>> {
///         Box::pin(async { vec![] })
///     }
/// }
/// ```
pub trait ReadResourceCallback: Send + Sync {
    /// Calls the read resource function with the given URI
    ///
    /// # Arguments
    ///
    /// * `uri` - The URI of the resource to read
    ///
    /// # Returns
    ///
    /// A future that resolves to the resource contents
    fn call(&self, uri: &Url) -> ReadResourceFuture;
}

/// Callback function for reading resources
///
/// Wraps a function that returns a future resolving to resource contents.
pub struct ReadResourceCallbackFn(pub ReadResourceFn);

impl ReadResourceCallback for ReadResourceCallbackFn {
    fn call(&self, uri: &Url) -> ReadResourceFuture {
        (self.0)(uri)
    }
}

/// A registered resource with metadata and callbacks
///
/// Represents a resource that has been registered with the server,
/// including its metadata, read callback, and update channel.
///
/// # Examples
///
/// ```rust
/// use url::Url;
/// use mcp_daemon::server::resource::RegisteredResource;
/// use mcp_daemon::types::{Resource, ResourceContents};
///
/// // Create a resource
/// let resource_uri = Url::parse("file:///example.txt").unwrap();
/// let resource = Resource {
///     uri: resource_uri.clone(),
///     name: "Example File".to_string(),
///     description: Some("An example text file".to_string()),
///     mime_type: Some("text/plain".to_string()),
/// };
///
/// // Register the resource with a read callback
/// let registered = RegisteredResource::new(
///     resource,
///     |uri: &Url| async move {
///         vec![ResourceContents {
///             uri: uri.clone(),
///             text: Some("This is the content of the example file.".to_string()),
///             blob: None,
///             mime_type: Some("text/plain".to_string()),
///         }]
///     },
///     true, // supports subscriptions
/// );
/// ```
pub(crate) struct RegisteredResource {
    /// The resource metadata
    #[allow(dead_code)]
    pub metadata: Resource,
    /// The callback to read the resource
    #[allow(dead_code)]
    pub read_callback: Arc<dyn ReadResourceCallback>,
    /// Channel for resource update notifications
    #[allow(dead_code)]
    pub update_channel: ResourceUpdateChannel,
    /// Whether this resource supports subscriptions
    #[allow(dead_code)]
    pub supports_subscriptions: bool,
}

impl RegisteredResource {
    /// Create a new registered resource
    ///
    /// # Arguments
    ///
    /// * `metadata` - The resource metadata
    /// * `read_callback` - The callback to read the resource
    /// * `supports_subscriptions` - Whether this resource supports subscriptions
    ///
    /// # Returns
    ///
    /// A new RegisteredResource instance
    ///
    /// # Examples
    ///
    /// ```rust
    /// use url::Url;
    /// use mcp_daemon::server::resource::RegisteredResource;
    /// use mcp_daemon::types::{Resource, ResourceContents};
    ///
    /// let resource_uri = Url::parse("file:///example.txt").unwrap();
    /// let resource = Resource {
    ///     uri: resource_uri.clone(),
    ///     name: "Example File".to_string(),
    ///     description: None,
    ///     mime_type: Some("text/plain".to_string()),
    /// };
    ///
    /// let registered = RegisteredResource::new(
    ///     resource,
    ///     |uri: &Url| async move {
    ///         vec![ResourceContents {
    ///             uri: uri.clone(),
    ///             text: Some("File content".to_string()),
    ///             blob: None,
    ///             mime_type: Some("text/plain".to_string()),
    ///         }]
    ///     },
    ///     true, // supports subscriptions
    /// );
    /// ```
    pub fn new(
        metadata: Resource,
        read_callback: impl Fn(&Url) -> ReadResourceFuture + Send + Sync + 'static,
        supports_subscriptions: bool,
    ) -> Self {
        Self {
            metadata,
            read_callback: Arc::new(ReadResourceCallbackFn(Box::new(read_callback))),
            update_channel: ResourceUpdateChannel::new(),
            supports_subscriptions,
        }
    }

    /// Subscribe to updates for this resource
    ///
    /// # Returns
    ///
    /// An optional broadcast receiver for update notifications
    #[allow(dead_code)]
    pub fn subscribe(&self) -> Option<broadcast::Receiver<String>> {
        if self.supports_subscriptions {
            Some(self.update_channel.subscribe(&self.metadata.uri))
        } else {
            None
        }
    }

    /// Unsubscribe from updates for this resource
    #[allow(dead_code)]
    pub fn unsubscribe(&self) {
        if self.supports_subscriptions {
            self.update_channel.unsubscribe(&self.metadata.uri);
        }
    }

    /// Notify subscribers that this resource has been updated
    #[allow(dead_code)]
    pub fn notify_update(&self) {
        if self.supports_subscriptions {
            self.update_channel.notify_update(&self.metadata.uri);
        }
    }
}

/// A registered resource template with metadata and callbacks
///
/// Represents a resource template that has been registered with the server,
/// including its metadata and read callback.
pub(crate) struct RegisteredResourceTemplate {
    /// The resource template
    #[allow(dead_code)]
    pub template: ResourceTemplate,
    /// The resource metadata
    #[allow(dead_code)]
    pub metadata: Resource,
    /// The callback to read resources matching the template
    #[allow(dead_code)]
    pub read_callback: ArcReadResourceCallback,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::completable::CompletableString;

    #[tokio::test]
    async fn test_resource_template() {
        let template = ResourceTemplate::new("file://{path}")
            .with_list(|| async { vec![] })
            .with_completion(
                "path",
                CompletableString::new(|input: &str| {
                    let input = input.to_string();
                    async move { vec![format!("{}/file.txt", input)] }
                }),
            );

        assert_eq!(template.uri_template(), "file://{path}");
        assert!(template.list_callback().is_some());
        assert!(template.complete_callback("path").is_some());
        assert!(template.complete_callback("nonexistent").is_none());
    }
}
