//! Binary telemetry protocol shared by the simulator and gateway.
//! Frame: magic(2) | version(1) | payload length(2, BE) | payload(12) | CRC32(4, BE).
//! The CRC covers the header and payload, but not the trailing checksum.

use std::fmt;
use std::io::{self, Read};


/*
+--------+---------+--------+----------+----------+-------------+----------+
| Magic  | Version | Length | Device ID| Sequence | Temperature | Checksum |
+--------+---------+--------+----------+----------+-------------+----------+
| 2 bytes| 1 byte  | 2 bytes| 4 bytes  | 4 bytes  | 4 bytes     | 4 bytes  |
+--------+---------+--------+----------+----------+-------------+----------+


+---------------------------+------------------------------------+---------+----------+
| Header                    | Payload                            | CRC32   | Total    |
+---------------------------+------------------------------------+---------+----------+
| Magic + Version + Length  | Device ID + Sequence + Temperature |         |          |
| 2 bytes + 1 byte + 2 bytes| 4 bytes + 4 bytes + 4 bytes        |         |          |
+---------------------------+------------------------------------+---------+----------+
| 5 bytes                   | 12 bytes                           | 4 bytes | 21 bytes |
+---------------------------+------------------------------------+---------+----------+
*/

const MAGIC: [u8; 2] = *b"RT";
const VERSION: u8 = 1;
const HEADER_LENGTH: usize = 5;
const CHECKSUM_LENGTH: usize = 4;
const TELEMETRY_LENGTH: usize = 12;
pub const MAX_PAYLOAD_LENGTH: usize = 1024;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Telemetry {
	pub device_id: u32,
	pub sequence: u32,
	pub temperature_c: f32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProtocolError {
	InvalidMagic,
	UnsupportedVersion(u8),
	OversizedPayload(usize),
	InvalidPayloadLength(usize),
	InvalidFrameLength(usize),
	ChecksumMismatch,
	NonFiniteTemperature,
}

impl fmt::Display for ProtocolError {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(formatter, "{self:?}")
	}
}

impl std::error::Error for ProtocolError {}

pub fn crc32(bytes: &[u8]) -> u32 {
	const POLYNOMIAL: u32 = 0xEDB88320;
	let mut crc: u32 = u32::MAX;

	for byte_reference in bytes {
		let byte: u8 = *byte_reference;
		crc ^= u32::from(byte);

		for _bit_index in 0u8..8u8 {
			let rightmost_bit: u32 = crc & 1u32;
			crc >>= 1;
			if rightmost_bit != 0u32 {
				crc ^= POLYNOMIAL;
			}
		}
	}

	!crc
}

// converting Telemetry fields into a sequence of bytes that can be transmitted over TCP.
pub fn encode(value: &Telemetry) -> Result<Vec<u8>, ProtocolError> {
	if !value.temperature_c.is_finite() {
		return Err(ProtocolError::NonFiniteTemperature);
	}

	let capacity: usize = HEADER_LENGTH + TELEMETRY_LENGTH + CHECKSUM_LENGTH;
	let mut frame: Vec<u8> = Vec::with_capacity(capacity);
	let payload_length: u16 = TELEMETRY_LENGTH as u16;

	frame.extend_from_slice(&MAGIC);
	frame.push(VERSION);
	frame.extend_from_slice(&payload_length.to_be_bytes());
	frame.extend_from_slice(&value.device_id.to_be_bytes());
	frame.extend_from_slice(&value.sequence.to_be_bytes());
	frame.extend_from_slice(&value.temperature_c.to_bits().to_be_bytes());

	let checksum: u32 = crc32(&frame);
	frame.extend_from_slice(&checksum.to_be_bytes());
	Ok(frame)
}

pub fn decode(frame: &[u8]) -> Result<Telemetry, ProtocolError> {
	if frame.len() < HEADER_LENGTH + CHECKSUM_LENGTH {
		return Err(ProtocolError::InvalidFrameLength(frame.len()));
	}
	if frame[..2] != MAGIC {
		return Err(ProtocolError::InvalidMagic);
	}
	if frame[2] != VERSION {
		return Err(ProtocolError::UnsupportedVersion(frame[2]));
	}

	let payload_length: usize = usize::from(u16::from_be_bytes([frame[3], frame[4]]));
	if payload_length > MAX_PAYLOAD_LENGTH {
		return Err(ProtocolError::OversizedPayload(payload_length));
	}
	if payload_length != TELEMETRY_LENGTH {
		return Err(ProtocolError::InvalidPayloadLength(payload_length));
	}

	let expected_length: usize = HEADER_LENGTH + payload_length + CHECKSUM_LENGTH;
	if frame.len() != expected_length {
		return Err(ProtocolError::InvalidFrameLength(frame.len()));
	}

	let checksum_offset: usize = HEADER_LENGTH + payload_length;
	let expected_checksum: u32 = u32::from_be_bytes([
		frame[checksum_offset],
		frame[checksum_offset + 1],
		frame[checksum_offset + 2],
		frame[checksum_offset + 3],
	]);
	let actual_checksum: u32 = crc32(&frame[..checksum_offset]);
	if expected_checksum != actual_checksum {
		return Err(ProtocolError::ChecksumMismatch);
	}

	let payload: &[u8] = &frame[HEADER_LENGTH..checksum_offset];
	let device_id: u32 = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
	let sequence: u32 = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);
	let temperature_bits: u32 = u32::from_be_bytes([payload[8], payload[9], payload[10], payload[11]]);
	let temperature_c: f32 = f32::from_bits(temperature_bits);

	if !temperature_c.is_finite() {
		return Err(ProtocolError::NonFiniteTemperature);
	}

	Ok(Telemetry {
		device_id,
		sequence,
		temperature_c,
	})
}

/// Reads exactly one frame; returns None only for EOF before the next frame.
pub fn read_frame<R: Read>(reader: &mut R) -> io::Result<Option<Telemetry>> {
	let mut first_byte: [u8; 1] = [0u8; 1];
	let first_read: usize = reader.read(&mut first_byte)?;

	if first_read == 0 {
		return Ok(None);
	}

	let mut header: [u8; HEADER_LENGTH] = [0u8; HEADER_LENGTH];

	header[0] = first_byte[0];
	reader.read_exact(&mut header[1..])?;

	if header[..2] != MAGIC {
		return Err(invalid_data(ProtocolError::InvalidMagic));
	}

	if header[2] != VERSION {
		return Err(invalid_data(ProtocolError::UnsupportedVersion(header[2])));
	}

	let payload_length: usize = usize::from(u16::from_be_bytes([header[3], header[4]]));

	if payload_length > MAX_PAYLOAD_LENGTH {
		return Err(invalid_data(ProtocolError::OversizedPayload(payload_length)));
	}

	if payload_length != TELEMETRY_LENGTH {
		return Err(invalid_data(ProtocolError::InvalidPayloadLength(payload_length)));
	}

	let frame_length: usize = HEADER_LENGTH + payload_length + CHECKSUM_LENGTH;
	let mut frame: Vec<u8> = vec![0u8; frame_length];

	frame[..HEADER_LENGTH].copy_from_slice(&header);
	reader.read_exact(&mut frame[HEADER_LENGTH..])?;

	let telemetry: Telemetry = decode(&frame)
		.map_err(invalid_data)?;

	Ok(Some(telemetry))
}

fn invalid_data(error: ProtocolError) -> io::Error {
	io::Error::new(io::ErrorKind::InvalidData, error)
}

#[cfg(test)]
mod tests {
	use super::{crc32, decode, encode, read_frame, ProtocolError, Telemetry};
	use std::io::{Cursor, ErrorKind, Read};

	fn sample() -> Telemetry {
		Telemetry {
			device_id: 7,
			sequence: 42,
			temperature_c: 23.5,
		}
	}

	#[test]
	fn roundtrip() {
		let frame: Vec<u8> = encode(&sample()).expect("encoding must succeed");
		let decoded: Telemetry = decode(&frame).expect("decoding must succeed");
		assert_eq!(decoded, sample());
	}

	#[test]
	fn known_crc() {
		let checksum: u32 = crc32(b"123456789");
		assert_eq!(checksum, 0xCBF43926);
	}

	#[test]
	fn corrupted_frame_is_rejected() {
		let mut frame: Vec<u8> = encode(&sample()).expect("encoding must succeed");
		frame[8] ^= 1u8;
		assert_eq!(decode(&frame), Err(ProtocolError::ChecksumMismatch));
	}

	#[test]
	fn invalid_magic_is_rejected() {
		let mut frame: Vec<u8> = encode(&sample()).expect("encoding must succeed");
		frame[0] = 0u8;
		assert_eq!(decode(&frame), Err(ProtocolError::InvalidMagic));
	}

	#[test]
	fn oversized_header_is_rejected_before_allocating() {
		let mut frame: Vec<u8> = encode(&sample()).expect("encoding must succeed");
		frame[3..5].copy_from_slice(&2048u16.to_be_bytes());
		let mut reader: Cursor<Vec<u8>> = Cursor::new(frame);
		let error_kind: ErrorKind = read_frame(&mut reader).unwrap_err().kind();
		assert_eq!(error_kind, ErrorKind::InvalidData);
	}

	#[test]
	fn truncated_frame_is_rejected() {
		let mut frame: Vec<u8> = encode(&sample()).expect("encoding must succeed");
		frame.pop();
		let mut reader: Cursor<Vec<u8>> = Cursor::new(frame);
		let error_kind: ErrorKind = read_frame(&mut reader).unwrap_err().kind();
		assert_eq!(error_kind, ErrorKind::UnexpectedEof);
	}

	#[test]
	fn concatenated_frames_are_read_separately() {
		let mut frames: Vec<u8> = encode(&sample()).expect("encoding must succeed");
		let second: Vec<u8> = encode(&sample()).expect("encoding must succeed");
		frames.extend_from_slice(&second);
		let mut reader: Cursor<Vec<u8>> = Cursor::new(frames);
		assert_eq!(read_frame(&mut reader).unwrap(), Some(sample()));
		assert_eq!(read_frame(&mut reader).unwrap(), Some(sample()));
		assert_eq!(read_frame(&mut reader).unwrap(), None);
	}

	struct OneByteReader<R: Read> {
		inner: R,
	}

	impl<R: Read> Read for OneByteReader<R> {
		fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
			let limit: usize = buffer.len().min(1usize);
			self.inner.read(&mut buffer[..limit])
		}
	}

	#[test]
	fn fragmented_reads_are_supported() {
		let frame: Vec<u8> = encode(&sample()).expect("encoding must succeed");
		let cursor: Cursor<Vec<u8>> = Cursor::new(frame);
		let mut reader: OneByteReader<Cursor<Vec<u8>>> = OneByteReader { inner: cursor };
		assert_eq!(read_frame(&mut reader).unwrap(), Some(sample()));
	}
}
