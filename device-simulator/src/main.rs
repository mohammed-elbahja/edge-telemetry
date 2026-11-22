use std::io::{self, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use telemetry_protocol::{encode, Telemetry};

fn main() -> io::Result<()> {
	let mut stream: TcpStream = TcpStream::connect("127.0.0.1:9000")?;
	stream.set_write_timeout(Some(Duration::from_secs(10)))?;
	println!("Connected to gateway");

	for sequence in 1u32..=5u32 {

		let measurement: Telemetry = Telemetry {
			device_id: 1,
			sequence,
			temperature_c: 21.0f32 + (sequence as f32) * 0.5f32,
		};

		let frame: Vec<u8> = encode(&measurement)
			.map_err(io::Error::other)?;

		let split_at: usize = 3;

		// Deliberately split a frame across two writes to exercise TCP framing.
		stream.write_all(&frame[..split_at])?;
		stream.write_all(&frame[split_at..])?;

		println!("Sent sequence={sequence} bytes={}", frame.len());

		thread::sleep(Duration::from_millis(500));
	}

	Ok(())
}
