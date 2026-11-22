use crate::pipeline::TelemetryPipeline;
use std::io;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;
use telemetry_protocol::{read_frame, Telemetry};

/// TCP transport and connection lifecycle; no telemetry business rules here.
pub struct GatewayRuntime {
	bind_address: String,
	read_timeout: Duration,
	pipeline: TelemetryPipeline,
}

impl GatewayRuntime {
	pub(crate) fn new(bind_address: String, read_timeout: Duration, pipeline: TelemetryPipeline) -> Self {
		Self {
			bind_address,
			read_timeout,
			pipeline,
		}
	}

	pub fn run(&self) -> io::Result<()> {
		let listener: TcpListener = TcpListener::bind(&self.bind_address)?;
		println!("Gateway listening on {}", listener.local_addr()?);

		for incoming in listener.incoming() {
			match incoming {
				Ok(stream) => {
					if let Err(error) = self.handle_client(stream) {
						eprintln!("Connection ended with error: {error}");
					}
				}
				Err(error) => eprintln!("TCP accept failed: {error}"),
			}
		}
		Ok(())
	}

	fn handle_client(&self, mut stream: TcpStream) -> io::Result<()> {
		let peer: SocketAddr = stream.peer_addr()?;
		stream.set_read_timeout(Some(self.read_timeout))?;
		println!("Connected: {peer}");

		loop {
			let received: Option<Telemetry> = read_frame(&mut stream)?;
			match received {
				Some(telemetry) => {
					// A rejected measurement ends this client's session, not the server.
					self.pipeline.execute(telemetry)?;
				}
				None => {
					println!("Disconnected: {peer}");
					return Ok(());
				}
			}
		}
	}
}
