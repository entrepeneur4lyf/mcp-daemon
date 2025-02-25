//! Completion system for intelligent argument auto-completion
//!
//! This module provides a framework for implementing auto-completion functionality
//! in the Model Context Protocol. It defines the `Completable` trait and several
//! implementations that can be used to provide completion suggestions for various
//! types of inputs.
//!
//! The completion system is designed to be:
//! - Asynchronous: Completion suggestions can be generated asynchronously
//! - Flexible: Different completion strategies can be implemented
//! - Type-safe: Completion suggestions are typed
//! - Extensible: New completion providers can be added easily
//!
//! # Examples
//!
//! ## Using CompletableString with a custom completion function
//!
//! ```
//! use mcp_daemon::completable::{Completable, CompletableString};
//!
//! async fn example() {
//!     // Create a completable string with a custom completion function
//!     let completable = CompletableString::new(|input: &str| {
//!         let input = input.to_string();
//!         async move {
//!             // Generate suggestions based on the input
//!             vec![
//!                 format!("{}1", input),
//!                 format!("{}2", input)
//!             ]
//!         }
//!     });
//!
//!     // Get completion suggestions
//!     let suggestions = completable.complete("test").await;
//!     assert_eq!(suggestions, vec!["test1", "test2"]);
//! }
//! ```
//!
//! ## Using FixedCompletions for predefined suggestions
//!
//! ```
//! use mcp_daemon::completable::{Completable, FixedCompletions};
//!
//! async fn example() {
//!     // Create a fixed completions provider with predefined values
//!     let completions = FixedCompletions::new(vec!["apple", "banana", "cherry"]);
//!
//!     // Get completion suggestions that match the input
//!     let suggestions = completions.complete("a").await;
//!     assert_eq!(suggestions, vec!["apple", "banana"]);
//! }
//! ```

use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::sync::Arc;
use std::fmt::Debug;

// Type aliases for complex future types
type CompletionFuture<T> = Pin<Box<dyn Future<Output = Vec<T>> + Send>>;
type CompletionFn<T> = Arc<dyn Fn(&str) -> CompletionFuture<T> + Send + Sync>;

/// A trait for types that can provide completion suggestions.
///
/// This trait defines the interface for types that can generate completion
/// suggestions for a given input. It is similar to the TypeScript SDK's
/// Completable type.
///
/// # Type Parameters
///
/// * `Input` - The input type for completion suggestions
/// * `Output` - The output type for completion suggestions
///
/// # Examples
///
/// ```
/// use mcp_daemon::completable::Completable;
/// use std::pin::Pin;
/// use std::future::Future;
///
/// struct MyCompletable;
///
/// impl Completable for MyCompletable {
///     type Input = str;
///     type Output = String;
///
///     fn complete(&self, value: &Self::Input) -> Pin<Box<dyn Future<Output = Vec<Self::Output>> + Send>> {
///         Box::pin(async move {
///             vec![format!("{}1", value), format!("{}2", value)]
///         })
///     }
/// }
///
/// # async fn example() {
/// let completable = MyCompletable;
/// let suggestions = completable.complete("test").await;
/// assert_eq!(suggestions, vec!["test1", "test2"]);
/// # }
/// ```
pub trait Completable {
    /// The input type for completion suggestions
    type Input: ?Sized + Debug;
    /// The output type for completion suggestions
    type Output;

    /// Generate completion suggestions for the given input value
    ///
    /// This method takes a reference to an input value and returns a future
    /// that resolves to a vector of completion suggestions.
    ///
    /// # Arguments
    ///
    /// * `value` - The input value to generate suggestions for
    ///
    /// # Returns
    ///
    /// A future that resolves to a vector of completion suggestions
    fn complete(&self, value: &Self::Input) -> CompletionFuture<Self::Output>;
}

/// A completable string that uses a callback function to generate suggestions
///
/// This type implements the `Completable` trait for strings, using a callback
/// function to generate completion suggestions. The callback function takes a
/// string reference and returns a future that resolves to a vector of strings.
///
/// # Examples
///
/// ```
/// use mcp_daemon::completable::{Completable, CompletableString};
///
/// # async fn example() {
/// // Create a completable string with a custom completion function
/// let completable = CompletableString::new(|input: &str| {
///     let input = input.to_string();
///     async move {
///         // Generate suggestions based on the input
///         vec![
///             format!("{}1", input),
///             format!("{}2", input)
///         ]
///     }
/// });
///
/// // Get completion suggestions
/// let suggestions = completable.complete("test").await;
/// assert_eq!(suggestions, vec!["test1", "test2"]);
/// # }
/// ```
pub struct CompletableString {
    complete_fn: CompletionFn<String>,
}

impl CompletableString {
    /// Create a new CompletableString with the given completion callback
    ///
    /// This constructor creates a new `CompletableString` with the given
    /// completion callback function. The callback function takes a string
    /// reference and returns a future that resolves to a vector of strings.
    ///
    /// # Arguments
    ///
    /// * `complete_fn` - The completion callback function
    ///
    /// # Returns
    ///
    /// A new `CompletableString` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_daemon::completable::CompletableString;
    ///
    /// // Create a completable string with a custom completion function
    /// let completable = CompletableString::new(|input: &str| {
    ///     let input = input.to_string();
    ///     async move {
    ///         // Generate suggestions based on the input
    ///         vec![
    ///             format!("{}1", input),
    ///             format!("{}2", input)
    ///         ]
    ///     }
    /// });
    /// ```
    pub fn new<F, Fut>(complete_fn: F) -> Self 
    where
        F: Fn(&str) -> Fut + Send + Sync + 'static,
        Fut: IntoFuture<Output = Vec<String>> + Send + 'static,
        Fut::IntoFuture: Send,
    {
        Self {
            complete_fn: Arc::new(move |input| {
                let input = input.to_string();
                Box::pin(complete_fn(&input).into_future())
            }),
        }
    }
}

impl Completable for CompletableString {
    type Input = str;
    type Output = String;

    /// Generate completion suggestions for the given input string
    ///
    /// This method delegates to the completion callback function to generate
    /// suggestions for the given input string.
    ///
    /// # Arguments
    ///
    /// * `value` - The input string to generate suggestions for
    ///
    /// # Returns
    ///
    /// A future that resolves to a vector of string suggestions
    fn complete(&self, value: &Self::Input) -> CompletionFuture<Self::Output> {
        (self.complete_fn)(value)
    }
}

/// A completable type that provides fixed suggestions
///
/// This type implements the `Completable` trait for any type that can be cloned,
/// sent across threads, and debugged. It provides a fixed set of suggestions
/// that are filtered based on the input string.
///
/// # Examples
///
/// ```
/// use mcp_daemon::completable::{Completable, FixedCompletions};
///
/// # async fn example() {
/// // Create a fixed completions provider with predefined values
/// let completions = FixedCompletions::new(vec!["apple", "banana", "cherry"]);
///
/// // Get completion suggestions that match the input
/// let suggestions = completions.complete("a").await;
/// assert_eq!(suggestions, vec!["apple", "banana"]);
/// # }
/// ```
pub struct FixedCompletions<T> {
    values: Vec<T>,
}

impl<T: Clone + Send + Debug + 'static> FixedCompletions<T> {
    /// Create a new FixedCompletions with the given values
    ///
    /// This constructor creates a new `FixedCompletions` with the given
    /// fixed set of values. These values will be filtered based on the
    /// input string when generating completion suggestions.
    ///
    /// # Arguments
    ///
    /// * `values` - The fixed set of values to provide as suggestions
    ///
    /// # Returns
    ///
    /// A new `FixedCompletions` instance
    ///
    /// # Examples
    ///
    /// ```
    /// use mcp_daemon::completable::FixedCompletions;
    ///
    /// // Create a fixed completions provider with predefined values
    /// let completions = FixedCompletions::new(vec!["apple", "banana", "cherry"]);
    /// ```
    pub fn new(values: Vec<T>) -> Self {
        Self { values }
    }
}

impl<T: Clone + Send + Debug + 'static> Completable for FixedCompletions<T> {
    type Input = str;
    type Output = T;
    
    /// Generate completion suggestions for the given input string
    ///
    /// This method filters the fixed set of values based on the input string
    /// and returns the matching values as suggestions. The filtering is done
    /// by converting both the input and the values to lowercase and checking
    /// if the value contains the input.
    ///
    /// # Arguments
    ///
    /// * `value` - The input string to filter suggestions by
    ///
    /// # Returns
    ///
    /// A future that resolves to a vector of matching suggestions
    fn complete(&self, value: &Self::Input) -> CompletionFuture<Self::Output> {
        let values = self.values.clone();
        let value = value.to_string();
        
        Box::pin(async move {
            values
                .into_iter()
                .filter(|v| format!("{:?}", v).to_lowercase().contains(&value.to_lowercase()))
                .collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_completable_string() {
        let completable = CompletableString::new(|input: &str| {
            let input = input.to_string();
            async move {
                vec![
                    format!("{}1", input),
                    format!("{}2", input)
                ]
            }
        });

        let suggestions = completable.complete("test").await;
        assert_eq!(suggestions, vec!["test1", "test2"]);
    }

    #[tokio::test]
    async fn test_fixed_completions() {
        let completions = FixedCompletions::new(vec!["apple", "banana", "cherry"]);
        let suggestions = completions.complete("a").await;
        assert_eq!(suggestions, vec!["apple", "banana"]);
    }
}
