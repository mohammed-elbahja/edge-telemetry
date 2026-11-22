use std::io;
use telemetry_protocol::Telemetry;

/// A destination for successfully processed measurements.
pub trait TelemetrySink: Send + Sync {
	fn write(&self, telemetry: &Telemetry) -> io::Result<()>;
}

pub struct ConsoleSink;

impl TelemetrySink for ConsoleSink {
	fn write(&self, telemetry: &Telemetry) -> io::Result<()> {
		println!(
			"device={} sequence={} temperature_c={:.1}",
			telemetry.device_id, telemetry.sequence, telemetry.temperature_c
		);
		Ok(())
	}
}
