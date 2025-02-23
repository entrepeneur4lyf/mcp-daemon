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
pub type ListResourcesResult = Vec<Resource>;
/// Result type for reading resources
pub type ReadResourceResult = Vec<ResourceContents>;

// Type aliases for complex Future types
type ListResourcesFuture = Pin<Box<dyn Future<Output = ListResourcesResult> + Send + 'static>>;
type ReadResourceFuture = Pin<Box<dyn Future<Output = ReadResourceResult> + Send + 'static>>;

// Type aliases for callback functions
type ListResourcesFn = Box<dyn Fn() -> ListResourcesFuture + Send + Sync>;
type ReadResourceFn = Box<dyn Fn(&Url) -> ReadResourceFuture + Send + Sync>;
type ArcReadResourceCallback = Arc<dyn ReadResourceCallback>;



/// A channel for resource update notifications
#[derive(Clone)]
pub struct ResourceUpdateChannel {
    /// The sender for resource updates
    sender: broadcast::Sender<String>,
    /// The set of subscribed URIs
    subscribed_uris: Arc<RwLock<HashSet<String>>>,
}

impl ResourceUpdateChannel {
    /// Create a new resource update channel
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self {
            sender,
            subscribed_uris: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Subscribe to updates for a resource
    pub fn subscribe(&self, uri: &Url) -> broadcast::Receiver<String> {
        self.subscribed_uris.write().unwrap().insert(uri.to_string());
        self.sender.subscribe()
    }

    /// Unsubscribe from updates for a resource
    pub fn unsubscribe(&self, uri: &Url) {
        self.subscribed_uris.write().unwrap().remove(&uri.to_string());
    }

    /// Send an update notification for a resource
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
    pub fn new(uri_template: impl Into<String>) -> Self {
        Self {
            uri_template: uri_template.into(),
            list_callback: None,
            complete_callbacks: HashMap::new(),
        }
    }

    /// Add a callback to list all resources matching this template
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
    pub fn uri_template(&self) -> &str {
        &self.uri_template
    }

    /// Get the list callback if one exists
    pub fn list_callback(&self) -> Option<&dyn ListResourcesCallback> {
        self.list_callback.as_deref()
    }

    /// Get the completion callback for a variable if one exists
    pub fn complete_callback(
        &self,
        variable: &str,
    ) -> Option<&dyn Completable<Input = str, Output = String>> {
        self.complete_callbacks.get(variable).map(|c| c.as_ref())
    }
}

/// A callback that can list resources matching a template
pub trait ListResourcesCallback: Send + Sync {
    /// Calls the list resources function
    fn call(&self) -> ListResourcesFuture;
}

/// Callback function for listing resources
pub struct ListResourcesCallbackFn(ListResourcesFn);

impl ListResourcesCallback for ListResourcesCallbackFn {
    fn call(&self) -> ListResourcesFuture {
        (self.0)()
    }
}

/// A callback that can read a resource
pub trait ReadResourceCallback: Send + Sync {
    /// Calls the read resource function with the given URI
    fn call(&self, uri: &Url) -> ReadResourceFuture;
}

/// Callback function for reading resources
pub struct ReadResourceCallbackFn(pub ReadResourceFn);

impl ReadResourceCallback for ReadResourceCallbackFn {
    fn call(&self, uri: &Url) -> ReadResourceFuture {
        (self.0)(uri)
    }
}

/// A registered resource with metadata and callbacks
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
