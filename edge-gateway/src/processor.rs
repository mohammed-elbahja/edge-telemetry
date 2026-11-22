use std::io;
use telemetry_protocol::Telemetry;

/// A stage may validate or transform telemetry before the next stage.
pub trait TelemetryProcessor: Send + Sync {
	fn process(&self, telemetry: Telemetry) -> io::Result<Telemetry>;
}

/// Rejects measurements outside a configured range.
pub struct TemperatureRangeValidator {
	minimum_c: f32,
	maximum_c: f32,
}

impl TemperatureRangeValidator {
	pub fn new(minimum_c: f32, maximum_c: f32) -> io::Result<Self> {
		if !minimum_c.is_finite() || !maximum_c.is_finite() || minimum_c > maximum_c {
			return Err(io::Error::new(
				io::ErrorKind::InvalidInput,
				"temperature limits must be finite and ordered",
			));
		}
		Ok(Self {
			minimum_c,
			maximum_c,
		})
	}
}

impl TelemetryProcessor for TemperatureRangeValidator {
	fn process(&self, telemetry: Telemetry) -> io::Result<Telemetry> {
		if telemetry.temperature_c < self.minimum_c
			|| telemetry.temperature_c > self.maximum_c
		{
			return Err(io::Error::new(
				io::ErrorKind::InvalidData,
				format!("temperature {} C is outside configured range", telemetry.temperature_c),
			));
		}
		Ok(telemetry)
	}
}

// pub struct SequenceValidator {
// 	sequence_tracking: HashMap<u32, u32>
// }
//
// impl SequenceValidator {
// 	pub fn new() -> io::Result<Self> {
// 		Ok(Self {
// 			sequence_tracking: HashMap::new()
// 		})
// 	}
// }
//
// impl TelemetryProcessor for SequenceValidator {
// 	fn process(&self, telemetry: Telemetry) -> io::Result<Telemetry> {
// 		panic!("sequence tracking is not implemented yet");
// 	}
// }
