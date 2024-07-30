Certainly! Here's the updated README that includes details on certain NIC chipsets and their limitations:

---

# YALT - Yet Another Load Tester

## Introduction

YALT (Yet Another Load Tester) is a command-line tool designed for performing load testing on TCP servers. It allows users to send a specified number of requests per second (RPS) to multiple IP addresses and ports, distributed based on a defined bias. The tool uses a token bucket algorithm to regulate the request rate and records the results in an SQLite database for further analysis.

## How It Works

YALT uses a token bucket algorithm to control the rate at which requests are sent to the target servers. The token bucket algorithm is configured with a rate (RPS) and a capacity, representing the maximum number of tokens the bucket can hold. Tokens are added to the bucket at the configured rate, and each request consumes a token. If there are no tokens available, the request is delayed until tokens become available.

The tool can distribute traffic to multiple target servers based on a defined bias. For example, if you specify two servers with a bias of 50 each, the traffic will be split evenly between them. YALT logs the success and failure of each request in an SQLite database, along with the timestamp, IP address, and payload.

## Usage

### Command-Line Interface

YALT is a command-line tool. You can run it with the following parameters:

- `--ips`: A list of target IP addresses and ports with their respective biases (e.g., `127.0.0.1:12345:50,127.0.0.1:12346:50`).
- `--rate`: The request rate in requests per second (e.g., `200`).
- `--duration`: The duration of the test in seconds (e.g., `30`).
- `--payload`: The payload to send with each request (e.g., `"Your payload here"`).

### Example

```sh
yalt --ips 127.0.0.1:12345:50,127.0.0.1:12346:50 --rate 200 --duration 30 --payload "Test payload"
```

This command will send 200 requests per second for 30 seconds, splitting the traffic evenly between the two specified IP addresses.

## Limitations

### Hardware Limitations

The performance of YALT is dependent on the hardware it runs on. Key hardware factors that affect performance include:

- **CPU**: Higher RPS requires more CPU resources to handle concurrent connections and data processing.
- **Memory**: Adequate memory is needed to manage connections, buffers, and data storage.
- **Network Interface**: The capacity of the network interface card (NIC) determines how much traffic can be handled. Ensure the NIC supports high throughput.

### TCP/IP Protocol Limitations

The TCP/IP protocol has inherent limitations that can affect high RPS:

- **Connection Overhead**: Establishing and closing TCP connections has overhead due to the three-way handshake and connection teardown processes.
- **Latency**: Network latency can impact the rate at which requests are processed and responses are received.
- **Port Exhaustion**: TCP connections use ephemeral ports. With a very high RPS, the available port range may be exhausted, leading to connection failures.
- **Congestion Control**: TCP’s congestion control mechanisms can reduce throughput if packet loss or high latency is detected.

To mitigate these limitations, consider the following:

- **Optimize Hardware**: Use powerful hardware with high CPU, ample memory, and a high-capacity NIC.
- **Tune TCP Settings**: Adjust TCP settings like `tcp_tw_reuse` and `tcp_tw_recycle` (if applicable) to handle TIME_WAIT state more efficiently.
- **Use Persistent Connections**: Where possible, reuse TCP connections to reduce the overhead of connection setup and teardown.

### NIC Chipset Limitations

Different NIC chipsets have varying capabilities and limitations. Here are some common NIC chipsets and their limitations:

#### Intel Ethernet Controller I350

- **Throughput**: Supports up to 1 Gbps per port.
- **Features**: Advanced features such as SR-IOV, QoS, and traffic management.
- **Limitations**: Limited by CPU and memory bandwidth when handling high packet rates. May require tuning for optimal performance.

#### Broadcom NetXtreme BCM5719

- **Throughput**: Supports up to 1 Gbps per port.
- **Features**: Integrated PHY, TCP segmentation offload, and checksum offload.
- **Limitations**: Limited hardware offloading capabilities compared to newer chipsets. May struggle with very high RPS.

#### Mellanox ConnectX-4

- **Throughput**: Supports up to 100 Gbps.
- **Features**: Advanced offloading capabilities, RDMA, and SR-IOV.
- **Limitations**: Requires proper driver and firmware versions for optimal performance. High power consumption.

#### Realtek RTL8111

- **Throughput**: Supports up to 1 Gbps.
- **Features**: Basic offloading features such as checksum offload and interrupt moderation.
- **Limitations**: Less efficient than Intel and Broadcom counterparts. Higher CPU utilization for high packet rates.

To achieve optimal performance, ensure your system is configured correctly and that the NIC driver and firmware are up to date.

## Integration Tests

YALT includes integration tests to verify its functionality under different load conditions. The integration tests are written in Rust and use the Tokio runtime for asynchronous execution.

### Running the Tests

To run the integration tests, use the following command:

```sh
cargo test -- --test-threads=1
```

### Test Scenarios

The integration tests cover various scenarios with different request rates and durations to ensure the robustness and reliability of YALT. Here are some of the test scenarios:

- Low RPS with short duration (10 seconds, 20 RPS)
- Medium RPS with short duration (10 seconds, 200 RPS)
- High RPS with short duration (10 seconds, 400 RPS)
- Very High RPS with short duration (10 seconds, 900 RPS)
- Low RPS with medium duration (30 seconds, 20 RPS)
- Medium RPS with medium duration (30 seconds, 200 RPS)
- High RPS with medium duration (30 seconds, 400 RPS)
- Very High RPS with medium duration (30 seconds, 900 RPS)
- Low RPS with long duration (60 seconds, 20 RPS)
- Medium RPS with long duration (60 seconds, 200 RPS)
- High RPS with long duration (60 seconds, 400 RPS)
- Very High RPS with long duration (60 seconds, 900 RPS)

### Test Implementation

The integration tests are implemented in the `tests/integration_test.rs` file. Each test sets up mock servers, runs the YALT application with specific parameters, and verifies the results recorded in the SQLite database.

Here's an example test function:

```rust
#[tokio::test]
async fn test_20_rps_10s() -> Result<(), Box<dyn std::error::Error>> {
    run_integration_test(20.0, 10).await
}
```

This test runs the YALT application with a request rate of 20 RPS for 10 seconds and verifies the results.
