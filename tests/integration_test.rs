use tempfile::TempDir;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::task;
use rusqlite::Connection;
use yalt::yalt::{Opt, run_yalt}; // Adjust the import path to match the module

// Test for 100:0 bias
#[tokio::test]
async fn test_100_0_bias() -> Result<(), Box<dyn std::error::Error>> {
    let ips = vec!["127.0.0.1:12345:100".to_string(), "127.0.0.1:12346:0".to_string()];
    run_integration_test(ips, 200.0, 10).await
}

// Test for 50:50 bias
#[tokio::test]
async fn test_50_50_bias() -> Result<(), Box<dyn std::error::Error>> {
    let ips = vec!["127.0.0.1:12345:50".to_string(), "127.0.0.1:12346:50".to_string()];
    run_integration_test(ips, 200.0, 10).await
}

// Test for 66:33 bias
#[tokio::test]
async fn test_66_33_bias() -> Result<(), Box<dyn std::error::Error>> {
    let ips = vec!["127.0.0.1:12345:66".to_string(), "127.0.0.1:12346:33".to_string()];
    run_integration_test(ips, 200.0, 10).await
}

// Standard tests
#[tokio::test]
async fn test_20_rps_10s() -> Result<(), Box<dyn std::error::Error>> {
    let ips = vec!["127.0.0.1:12345:50".to_string(), "127.0.0.1:12346:50".to_string()];
    run_integration_test(ips, 20.0, 10).await
}

#[tokio::test]
async fn test_200_rps_10s() -> Result<(), Box<dyn std::error::Error>> {
    let ips = vec!["127.0.0.1:12345:50".to_string(), "127.0.0.1:12346:50".to_string()];
    run_integration_test(ips, 200.0, 10).await
}

#[tokio::test]
async fn test_400_rps_10s() -> Result<(), Box<dyn std::error::Error>> {
    let ips = vec!["127.0.0.1:12345:50".to_string(), "127.0.0.1:12346:50".to_string()];
    run_integration_test(ips, 400.0, 10).await
}

#[tokio::test]
async fn test_900_rps_10s() -> Result<(), Box<dyn std::error::Error>> {
    let ips = vec!["127.0.0.1:12345:50".to_string(), "127.0.0.1:12346:50".to_string()];
    run_integration_test(ips, 900.0, 10).await
}

// Additional tests with varied durations
#[tokio::test]
async fn test_20_rps_30s() -> Result<(), Box<dyn std::error::Error>> {
    let ips = vec!["127.0.0.1:12345:50".to_string(), "127.0.0.1:12346:50".to_string()];
    run_integration_test(ips, 20.0, 30).await
}

#[tokio::test]
async fn test_200_rps_30s() -> Result<(), Box<dyn std::error::Error>> {
    let ips = vec!["127.0.0.1:12345:50".to_string(), "127.0.0.1:12346:50".to_string()];
    run_integration_test(ips, 200.0, 30).await
}

#[tokio::test]
async fn test_400_rps_30s() -> Result<(), Box<dyn std::error::Error>> {
    let ips = vec!["127.0.0.1:12345:50".to_string(), "127.0.0.1:12346:50".to_string()];
    run_integration_test(ips, 400.0, 30).await
}

#[tokio::test]
async fn test_900_rps_30s() -> Result<(), Box<dyn std::error::Error>> {
    let ips = vec!["127.0.0.1:12345:50".to_string(), "127.0.0.1:12346:50".to_string()];
    run_integration_test(ips, 900.0, 30).await
}

#[tokio::test]
async fn test_20_rps_60s() -> Result<(), Box<dyn std::error::Error>> {
    let ips = vec!["127.0.0.1:12345:50".to_string(), "127.0.0.1:12346:50".to_string()];
    run_integration_test(ips, 20.0, 60).await
}

#[tokio::test]
async fn test_200_rps_60s() -> Result<(), Box<dyn std::error::Error>> {
    let ips = vec!["127.0.0.1:12345:50".to_string(), "127.0.0.1:12346:50".to_string()];
    run_integration_test(ips, 200.0, 60).await
}

#[tokio::test]
async fn test_400_rps_60s() -> Result<(), Box<dyn std::error::Error>> {
    let ips = vec!["127.0.0.1:12345:50".to_string(), "127.0.0.1:12346:50".to_string()];
    run_integration_test(ips, 400.0, 60).await
}

#[tokio::test]
async fn test_900_rps_60s() -> Result<(), Box<dyn std::error::Error>> {
    let ips = vec!["127.0.0.1:12345:50".to_string(), "127.0.0.1:12346:50".to_string()];
    run_integration_test(ips, 900.0, 60).await
}

async fn run_integration_test(ips: Vec<String>, rate: f64, duration: u64) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("metrics.db");

    let payload = "Test payload";

    // Start mock servers
    let servers = start_mock_servers(&ips).await?;

    // Run the yalt application
    let opts = Opt {
        ips,
        rate,
        duration,
        payload: payload.to_string(),
    };
    run_yalt(opts, &db_path).await?;

    // Stop mock servers
    for server in servers {
        server.abort();
    }

    // Verify the results in the SQLite database
    let conn = Connection::open(&db_path)?;
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM metrics WHERE success = 1")?;
    let success_count: i64 = stmt.query_row([], |row| row.get(0))?;
    let expected_count = (rate as i64) * (duration as i64);
    // Allow a buffer of ±10%
    assert!(success_count >= expected_count * 90 / 100 && success_count <= expected_count * 110 / 100);

    Ok(())
}

async fn start_mock_servers(ips: &[String]) -> Result<Vec<task::JoinHandle<()>>, Box<dyn std::error::Error>> {
    let mut handles = Vec::new();
    for ip in ips {
        let addr = ip.split(':').take(2).collect::<Vec<_>>().join(":");
        let listener = TcpListener::bind(&addr).await?;
        let handle = task::spawn(async move {
            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut buffer = [0; 1024];
                let _ = socket.read(&mut buffer).await;
                let _ = socket.write_all(&buffer).await;
            }
        });
        handles.push(handle);
    }
    Ok(handles)
}
