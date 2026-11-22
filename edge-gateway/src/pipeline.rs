use crate::processor::TelemetryProcessor;
use crate::sink::TelemetrySink;
use std::io;
use telemetry_protocol::Telemetry;

/// Runs ordered stages, then sends the result to a destination.
/// Box<dyn Trait> provides runtime-selectable implementations.
pub struct TelemetryPipeline {
	processors: Vec<Box<dyn TelemetryProcessor>>,
	sink: Box<dyn TelemetrySink>,
}

impl TelemetryPipeline {
	pub fn new(processors: Vec<Box<dyn TelemetryProcessor>>, sink: Box<dyn TelemetrySink>) -> Self {
		Self { processors, sink }
	}

	pub fn execute(&self, telemetry: Telemetry) -> io::Result<()> {
		let mut current: Telemetry = telemetry;

		for processor in &self.processors {
			current = processor.process(current)?;
		}

		self.sink.write(&current)
	}
}

#[cfg(test)]
mod tests {
	use super::TelemetryPipeline;
	use crate::processor::{TelemetryProcessor, TemperatureRangeValidator};
	use crate::sink::TelemetrySink;
	use std::io;
	use std::sync::{Arc, Mutex};
	use telemetry_protocol::Telemetry;

	struct OffsetProcessor {
		offset_c: f32,
	}

	impl TelemetryProcessor for OffsetProcessor {
		fn process(&self, mut telemetry: Telemetry) -> io::Result<Telemetry> {
			telemetry.temperature_c += self.offset_c;
			Ok(telemetry)
		}
	}

	struct RecordingSink {
		measurements: Arc<Mutex<Vec<Telemetry>>>,
	}

	impl TelemetrySink for RecordingSink {
		fn write(&self, telemetry: &Telemetry) -> io::Result<()> {
			let mut guard: std::sync::MutexGuard<'_, Vec<Telemetry>> = self
				.measurements
				.lock()
				.map_err(|error: std::sync::PoisonError<std::sync::MutexGuard<'_, Vec<Telemetry>>>| {
					io::Error::other(error.to_string())
				})?;
			guard.push(*telemetry);
			Ok(())
		}
	}

	#[test]
	fn stages_run_in_order_and_sink_receives_result() {
		let measurements: Arc<Mutex<Vec<Telemetry>>> = Arc::new(Mutex::new(Vec::new()));
		let processors: Vec<Box<dyn TelemetryProcessor>> = vec![
			Box::new(OffsetProcessor { offset_c: 5.0 }),
			Box::new(TemperatureRangeValidator::new(20.0, 30.0).unwrap()),
		];
		let sink: Box<dyn TelemetrySink> = Box::new(RecordingSink {
			measurements: Arc::clone(&measurements),
		});
		let pipeline: TelemetryPipeline = TelemetryPipeline::new(processors, sink);
		let input: Telemetry = Telemetry {
			device_id: 1,
			sequence: 2,
			temperature_c: 18.0,
		};

		pipeline.execute(input).unwrap();

		let guard: std::sync::MutexGuard<'_, Vec<Telemetry>> = measurements.lock().unwrap();
		assert_eq!(guard.len(), 1);
		assert_eq!(guard[0].temperature_c, 23.0);
	}

	#[test]
	fn rejection_prevents_sink_write() {
		let measurements: Arc<Mutex<Vec<Telemetry>>> = Arc::new(Mutex::new(Vec::new()));
		let processors: Vec<Box<dyn TelemetryProcessor>> = vec![
			Box::new(TemperatureRangeValidator::new(0.0, 10.0).unwrap()),
		];
		let sink: Box<dyn TelemetrySink> = Box::new(RecordingSink {
			measurements: Arc::clone(&measurements),
		});
		let pipeline: TelemetryPipeline = TelemetryPipeline::new(processors, sink);
		let input: Telemetry = Telemetry {
			device_id: 1,
			sequence: 2,
			temperature_c: 18.0,
		};

		assert!(pipeline.execute(input).is_err());
		let guard: std::sync::MutexGuard<'_, Vec<Telemetry>> = measurements.lock().unwrap();
		assert!(guard.is_empty());
	}
}
