//! Gateway components are intentionally synchronous in Milestone 003.
//! Traits isolate application policy from TCP and binary framing.

pub mod builder;
pub mod pipeline;
pub mod processor;
pub mod runtime;
pub mod sink;

pub use builder::GatewayBuilder;
pub use pipeline::TelemetryPipeline;
pub use processor::{TemperatureRangeValidator, TelemetryProcessor};
pub use runtime::GatewayRuntime;
pub use sink::{ConsoleSink, TelemetrySink};
