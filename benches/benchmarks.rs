//! Benchmarks for mcp_daemon
//! 
//! This module provides performance benchmarks for key components of the mcp_daemon crate.
//! Benchmarks are organized into logical groups for better comparison and analysis.

use mcp_daemon::{
    completable::{Completable, CompletableString, FixedCompletions},
    server::{
        notifications::{CancelledParams, Notification, NotificationSender},
        prompt::{GetPromptResult, PromptBuilder},
        Server,
    },
    transport::{Message, Result as TransportResult, Transport},
    types::{Implementation, ServerCapabilities},
};
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use std::time::Duration;
use tokio::runtime::Runtime;
use async_trait::async_trait;
use serde_json;

/// Helper to run async benchmarks consistently
fn bench_async<F, Fut>(rt: &Runtime, f: F) -> Fut::Output
where
    F: Fn() -> Fut,
    Fut: std::future::Future,
{
    rt.block_on(f())
}

// Zero-overhead mock transport for consistent benchmarking
#[derive(Clone)]
struct MockTransport;

#[async_trait]
impl Transport for MockTransport {
    async fn receive(&self) -> TransportResult<Option<Message>> { Ok(None) }
    async fn send(&self, _: &Message) -> TransportResult<()> { Ok(()) }
    async fn open(&self) -> TransportResult<()> { Ok(()) }
    async fn close(&self) -> TransportResult<()> { Ok(()) }
}

// Zero-overhead mock notification sender
#[derive(Clone)]
struct MockNotificationSender;

#[async_trait]
impl NotificationSender for MockNotificationSender {
    async fn send(&self, _: Notification) -> Result<(), mcp_daemon::server::error::ServerError> {
        Ok(())
    }
}

/// Benchmark completion-related functionality
fn completions(c: &mut Criterion) {
    let mut group = c.benchmark_group("completions");
    group.throughput(Throughput::Elements(1));
    group.warm_up_time(Duration::from_secs(1));

    // Pre-allocate completable with static callback
    let completable = CompletableString::new(|input: &str| {
        let input = input.to_string();
        async move { vec![format!("{input}1"), format!("{input}2")] }
    });

    let rt = Runtime::new().unwrap();
    group.bench_function("string", |b| {
        b.iter(|| bench_async(&rt, || completable.complete("test")))
    });

    // Pre-allocate fixed completions
    let fixed = FixedCompletions::new(vec!["apple", "banana", "cherry"]);
    group.bench_function("fixed", |b| {
        b.iter(|| bench_async(&rt, || fixed.complete("a")))
    });

    group.finish();
}

/// Benchmark notification handling
fn notifications(c: &mut Criterion) {
    let mut group = c.benchmark_group("notifications");
    group.throughput(Throughput::Elements(1));
    group.warm_up_time(Duration::from_secs(1));
    
    // Pre-allocate server and runtime
    let rt = Runtime::new().unwrap();
    let mut server = Server::new(Implementation {
        name: "test".into(),
        version: "0.1.0".into(),
    });
    server.set_notification_sender(MockNotificationSender);

    // Pre-allocate notification
    let notification = Notification::Initialized;

    group.bench_function("send", |b| {
        b.iter_batched(
            || notification.clone(),
            |n| bench_async(&rt, || server.send_notification(n.clone())),
            BatchSize::SmallInput,
        )
    });

    // Benchmark notification serialization
    let cancelled = Notification::Cancelled(CancelledParams {
        request_id: "123".into(),
        reason: Some("User cancelled".into()),
    });

    group.bench_function("serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&cancelled)))
    });

    group.finish();
}

/// Benchmark prompt building and handling
fn prompt(c: &mut Criterion) {
    let mut group = c.benchmark_group("prompt");
    group.throughput(Throughput::Elements(1));
    group.warm_up_time(Duration::from_secs(1));

    // Pre-allocate strings and callback
    let name = "test";
    let desc = "A test prompt";
    let arg1 = "arg1";
    let arg1_desc = "First argument";
    let arg2 = "arg2";

    group.bench_function("builder", |b| {
        b.iter(|| {
            PromptBuilder::new(black_box(name))
                .description(black_box(desc))
                .required_arg(black_box(arg1), Some(black_box(arg1_desc)))
                .optional_arg(black_box(arg2), None)
                .build(|_| async { GetPromptResult::default() })
        })
    });

    group.finish();
}

/// Benchmark server operations
fn server(c: &mut Criterion) {
    let mut group = c.benchmark_group("server");
    group.throughput(Throughput::Elements(1));
    group.warm_up_time(Duration::from_secs(1));

    // Pre-allocate implementation and transport
    let impl_ = Implementation {
        name: "test".into(),
        version: "0.1.0".into(),
    };
    let transport = MockTransport;
    let rt = Runtime::new().unwrap();

    group.bench_function("connect", |b| {
        b.iter_batched(
            || (Server::new(impl_.clone()), transport.clone()),
            |(server, t)| bench_async(&rt, || server.connect(t.clone())),
            BatchSize::SmallInput,
        )
    });

    // Pre-allocate server and capabilities
    let mut server = Server::new(impl_);
    let capabilities = ServerCapabilities::default();

    group.bench_function("capabilities", |b| {
        b.iter_batched(
            || capabilities.clone(),
            |caps| server.register_capabilities(caps),
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(
    benches,
    completions,
    notifications,
    prompt,
    server
);
criterion_main!(benches);
