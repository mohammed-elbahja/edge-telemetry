use crate::pipeline::TelemetryPipeline;
use crate::processor::TelemetryProcessor;
use crate::runtime::GatewayRuntime;
use crate::sink::{ConsoleSink, TelemetrySink};
use std::io;
use std::time::Duration;

/// Configures the runtime without coupling it to concrete processing stages.
pub struct GatewayBuilder {
	bind_address: String,
	read_timeout: Duration,
	processors: Vec<Box<dyn TelemetryProcessor>>,
	sink: Box<dyn TelemetrySink>,
}

impl GatewayBuilder {
	pub fn new() -> Self {
		Self {
			bind_address: String::from("127.0.0.1:9000"),
			read_timeout: Duration::from_secs(10),
			processors: Vec::new(),
			sink: Box::new(ConsoleSink),
		}
	}

	pub fn bind_address(mut self, address: &str) -> Self {
		self.bind_address = String::from(address);
		self
	}

	pub fn read_timeout(mut self, timeout: Duration) -> Self {
		self.read_timeout = timeout;
		self
	}

	pub fn add_processor<P: TelemetryProcessor + 'static>(mut self, processor: P) -> Self {
		self.processors.push(Box::new(processor));
		self
	}

	pub fn sink<S: TelemetrySink + 'static>(mut self, sink: S) -> Self {
		self.sink = Box::new(sink);
		self
	}

	pub fn build(self) -> io::Result<GatewayRuntime> {
		if self.bind_address.trim().is_empty() || self.read_timeout.is_zero() {
			return Err(io::Error::new(
				io::ErrorKind::InvalidInput,
				"bind address must not be empty and read timeout must be positive",
			));
		}
		let pipeline: TelemetryPipeline = TelemetryPipeline::new(self.processors, self.sink);
		Ok(GatewayRuntime::new(self.bind_address, self.read_timeout, pipeline))
	}
}

impl Default for GatewayBuilder {
	fn default() -> Self {
		Self::new()
	}
}
