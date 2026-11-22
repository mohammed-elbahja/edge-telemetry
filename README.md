# Edge Telemetry

A hands-on systems programming project exploring edge telemetry, binary protocols, network communication, memory management, and concurrency through incremental development.

**A hands-on Rust systems engineering project for building a reliable, extensible edge telemetry gateway from the ground up.**

Edge Telemetry is a progressive implementation of a network-based telemetry ingestion system written in Rust. The project explores how edge devices communicate with gateway services, how binary protocols are designed and validated, and how telemetry can be received, processed, and eventually delivered to downstream systems.

Rather than relying entirely on existing networking and messaging frameworks, the initial implementation focuses on building and understanding the fundamental components involved in reliable network communication and telemetry processing.

The project is developed incrementally, with each milestone introducing new capabilities while preserving the correctness and maintainability of previous implementations.

## 1. Project Objectives

The primary objective is to gain practical experience with Rust while designing and implementing meaningful systems engineering components.

The project focuses on:

- Rust ownership, borrowing, lifetimes, and memory management.
- TCP networking and asynchronous I/O.
- Binary protocol design, encoding, and decoding.
- Network framing and partial-read handling.
- Data integrity and protocol validation.
- Stateful telemetry processing.
- Device identification and sequence tracking.
- Error handling and fault isolation.
- Concurrency and resource management.
- Modular architecture and reusable components.
- Automated testing and verification.
- Extensible industrial connectivity.

The long-term objective is to evolve the project into a modular edge telemetry system capable of supporting multiple device protocols, transport mechanisms, and downstream data processing integrations.

## 2. Architecture

The initial architecture consists of three primary components.

```text
+------------------------+
|    Device Simulator    |
|------------------------|
| Telemetry Generation   |
| Sequence Numbers       |
| Binary Encoding        |
| TCP Client             |
+-----------+------------+
            |
            | TCP
            | Binary Telemetry Frames
            v
+------------------------+
|      Edge Gateway      |
|------------------------|
| TCP Listener           |
| Connection Handling    |
| Frame Reception        |
| Binary Decoding        |
| Protocol Validation    |
| Telemetry Processing   |
+-----------+------------+
            |
            v
+------------------------+
|   Telemetry Protocol   |
|------------------------|
| Frame Definitions      |
| Encoding / Decoding    |
| CRC32 Verification     |
| Protocol Errors        |
+------------------------+
```

- The device simulator generates telemetry messages and transmits them to the gateway using a custom binary protocol over TCP.
- The gateway receives the incoming byte stream, reconstructs complete frames, validates their integrity, and extracts telemetry data for processing.
- The shared protocol library defines the binary message format and the rules governing encoding, decoding, and validation.

The architecture is intentionally modular to allow additional protocols, transports, processors, and integrations to be introduced without tightly coupling them to the networking layer.

## 3. Repository Structure

### telemetry-protocol

A reusable Rust library responsible for defining and handling the binary telemetry protocol.

Responsibilities include:

- Frame encoding and decoding.
- Protocol version verification.
- Message-length validation.
- CRC32 integrity verification.
- Payload extraction.
- Protocol-specific error reporting.

### device-simulator

A simulated telemetry device used to generate and transmit telemetry messages.

It provides a controlled environment for testing network communication and gateway behavior without requiring physical devices or electronics.

### edge-gateway

The network-facing component responsible for accepting telemetry connections and processing incoming messages.

Its responsibilities include:

- Listening for TCP connections.
- Receiving telemetry byte streams.
- Reconstructing binary frames.
- Validating protocol messages.
- Passing decoded telemetry to processing components.

## 4. Binary Telemetry Protocol

The initial implementation uses a compact binary protocol with a fixed-length telemetry payload. Each frame contains a header, telemetry payload, and checksum.

### Frame Layout

| Field | Size | Description |
|---|---:|---|
| Magic | 2 bytes | Protocol identification |
| Version | 1 byte | Protocol version |
| Payload Length | 2 bytes | Length of the payload |
| Device ID | 4 bytes | Unique device identifier |
| Sequence Number | 4 bytes | Telemetry message sequence |
| Temperature | 4 bytes | IEEE 754 floating-point value |
| CRC32 | 4 bytes | Frame integrity checksum |
| **Total** | **21 bytes** | Complete telemetry frame |

Multi-byte numeric fields are encoded using big-endian byte order.

### Frame Representation

```text
+---------------------------+
| Header                    |
|---------------------------|
| Magic              2 bytes|
| Version            1 byte |
| Payload Length     2 bytes|
+---------------------------+
| Payload                   |
|---------------------------|
| Device ID          4 bytes|
| Sequence Number    4 bytes|
| Temperature        4 bytes|
+---------------------------+
| Checksum                  |
|---------------------------|
| CRC32              4 bytes|
+---------------------------+
```

### Protocol Validation

The protocol implementation includes validation for:

- Invalid magic values.
- Unsupported protocol versions.
- Invalid payload lengths.
- Truncated frames.
- CRC32 mismatches.
- Non-finite temperature values.

TCP delivers a continuous byte stream rather than preserving application-level message boundaries. The implementation therefore explicitly handles fragmented reads and reconstructs complete frames before decoding.

## 5. Telemetry Processing

The telemetry-processing design separates message decoding from application-level validation. This allows additional processing rules to be introduced without modifying the underlying binary protocol.

### Sequence Validation

Sequence numbers allow the gateway to track message ordering independently for each device. The intended validation behavior includes identifying:

- Duplicate sequence numbers.
- Out-of-order messages.
- Unexpected sequence gaps.
- Sequence tracking on a per-device basis.

### Temperature Validation

Temperature validation introduces application-level rules beyond binary protocol correctness. For the initial temperature telemetry use case, the accepted range is:

```text
Minimum: -40 °C
Maximum: 125 °C
```

Values outside this range are rejected by the application-level validator. These rules are separate from the protocol decoder, which is responsible for interpreting the binary representation and verifying structural integrity.

## 6. Implementation Progress

Development follows incremental milestones.

Each milestone introduces new engineering concepts and functional capabilities.

| Milestone | Focus |
|---|---|
| M001 | TCP communication and initial device simulation |
| M002 | Binary telemetry framing, encoding, and decoding |
| M003 | Protocol integrity, validation, and stream handling |
| Next | Stateful sequence validation and telemetry-processing improvements |
| Future | Multiple device connections and concurrency |
| Future | Backpressure and resource management |
| Future | Additional industrial protocols and transports |
| Future | Persistent telemetry pipelines and downstream integrations |

The roadmap is intentionally evolutionary.

## 7. Building and Running

### Prerequisites

- Rust toolchain compatible with `rust-toolchain.toml`.
- Cargo.
- A supported development environment such as Linux or WSL.

### Clone the Repository

```bash
git clone https://github.com/mohammed-elbahja/edge-telemetry.git

cd edge-telemetry
```

### Build the Workspace

```bash
cargo build --workspace
```

### Run the Tests

```bash
cargo test --workspace
```

### Validate Formatting

```bash
cargo fmt --all -- --check
```

### Run Clippy

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

### Run the Gateway

```bash
cargo run -p edge-gateway
```

### Run the Device Simulator

In another terminal:

```bash
cargo run -p device-simulator
```

The gateway should be started before the simulator so that it is available to accept incoming TCP connections.

Depending on the current milestone, network address and port settings may be defined in the application configuration or source code.

## 8. Testing Strategy

Testing is an integral part of the implementation. The project emphasizes verifying both successful operation and failure behavior.

### Protocol Tests

The protocol test suite covers scenarios such as:

- Encoding and decoding valid telemetry frames.
- CRC32 verification.
- Detecting corrupted messages.
- Rejecting malformed headers.
- Handling truncated input.
- Processing consecutive frames.
- Reconstructing frames received through fragmented TCP reads.

### Planned Testing Improvements

Future testing work includes:

- Stateful sequence-validation tests.
- Boundary-value tests for telemetry.
- Property-based testing.
- Fuzz testing of binary decoders.
- Concurrent device simulation.
- Network disconnection and reconnection scenarios.
- Resource-exhaustion and backpressure tests.
- Long-running gateway stability tests.

The objective is to verify system behavior under realistic operating conditions, including malformed input and network failures.

## 9. Engineering Principles

The implementation follows several engineering principles.

### Correctness Before Complexity

Each component should behave correctly under its defined operating conditions before introducing additional functionality.

### Explicit Resource Management

Memory allocation, ownership, connection lifetimes, and shared state are treated as architectural concerns rather than implementation details to address later.

### Separation of Responsibilities

Transport handling, binary protocol processing, telemetry validation, and downstream processing should remain independently understandable and testable.

### Defensive Input Handling

Incoming network data is treated as untrusted.

Frame lengths, checksums, protocol versions, and payload values must be validated before the data is accepted for further processing.

### Extensibility Without Premature Abstraction

The architecture should support future requirements without introducing unnecessary abstraction layers before their responsibilities are understood.

### Progressive Implementation

New capabilities are introduced through meaningful implementations, supported by tests and documented design decisions.

## 10. Future Directions

The longer-term roadmap includes exploring additional capabilities across several areas.

**Industrial Connectivity**

- Modbus TCP.
- MQTT.
- UDP telemetry.
- Serial communication.
- Protocol adapters and device-specific integrations.

**Telemetry Processing**

- Configurable processing pipelines.
- Composable telemetry processors.
- Device state management.
- Data validation and normalization.
- Stateful processing and deduplication.

**Networking and Reliability**

- Concurrent device connections.
- Connection lifecycle management.
- Bounded queues and backpressure.
- Reconnection and recovery strategies.
- Resource limits and admission control.

**Data Integration**

- Persistent telemetry storage.
- Downstream event publication.
- Integration with message brokers.
- Edge-to-cloud telemetry delivery.
- Local buffering for disconnected operation.

**Observability**

- Structured logging.
- Operational metrics.
- Connection and device diagnostics.
- Processing error visibility.
- Performance measurements and benchmarking.

These capabilities represent areas for future development and experimentation, not features currently guaranteed by the project.

## 11. Project Scope

Edge Telemetry is an independent systems-engineering and Rust learning project.

The purpose is to develop a solid understanding of the engineering foundations required to build reliable telemetry and connectivity software.

The implementation may evolve toward more advanced edge-processing and industrial connectivity capabilities as the project progresses.

## 12. License

Licensed under the Apache License, Version 2.0.

See [LICENSE](LICENSE) for details.

---

**Author:** Mohammed El Bahja

**GitHub:** [mohammed-elbahja](https://github.com/mohammed-elbahja)