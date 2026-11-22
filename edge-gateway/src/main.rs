use edge_gateway::{GatewayBuilder, GatewayRuntime, TemperatureRangeValidator};
use std::io;
use std::time::Duration;

fn main() -> io::Result<()> {
	let validator: TemperatureRangeValidator = TemperatureRangeValidator::new(-40.0, 125.0)?;
	
	let gateway: GatewayRuntime = GatewayBuilder::new()
		.bind_address("127.0.0.1:9000")
		.read_timeout(Duration::from_secs(10))
		.add_processor(validator)
		.build()?;

	gateway.run()
}