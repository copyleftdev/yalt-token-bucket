use tempfile::TempDir;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::task;
use rusqlite::Connection;
use yalt::yalt::{Opt, run_yalt}; // Adjust the import path to match the module

#[tokio::test]
async fn test_20_rps_10s() -> Result<(), Box<dyn std::error::Error>> {
    run_integration_test(20.0, 10).await
}

#[tokio::test]
async fn test_200_rps_10s() -> Result<(), Box<dyn std::error::Error>> {
    run_integration_test(200.0, 10).await
}

#[tokio::test]
async fn test_400_rps_10s() -> Result<(), Box<dyn std::error::Error>> {
    run_integration_test(400.0, 10).await
}

#[tokio::test]
async fn test_900_rps_10s() -> Result<(), Box<dyn std::error::Error>> {
    run_integration_test(900.0, 10).await
}

#[tokio::test]
async fn test_20_rps_30s() -> Result<(), Box<dyn std::error::Error>> {
    run_integration_test(20.0, 30).await
}

#[tokio::test]
async fn test_200_rps_30s() -> Result<(), Box<dyn std::error::Error>> {
    run_integration_test(200.0, 30).await
}

#[tokio::test]
async fn test_400_rps_30s() -> Result<(), Box<dyn std::error::Error>> {
    run_integration_test(400.0, 30).await
}

#[tokio::test]
async fn test_900_rps_30s() -> Result<(), Box<dyn std::error::Error>> {
    run_integration_test(900.0, 30).await
}

#[tokio::test]
async fn test_20_rps_60s() -> Result<(), Box<dyn std::error::Error>> {
    run_integration_test(20.0, 60).await
}

#[tokio::test]
async fn test_200_rps_60s() -> Result<(), Box<dyn std::error::Error>> {
    run_integration_test(200.0, 60).await
}

#[tokio::test]
async fn test_400_rps_60s() -> Result<(), Box<dyn std::error::Error>> {
    run_integration_test(400.0, 60).await
}

#[tokio::test]
async fn test_900_rps_60s() -> Result<(), Box<dyn std::error::Error>> {
    run_integration_test(900.0, 60).await
}

async fn run_integration_test(rate: f64, duration: u64) -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("metrics.db");

    let ip1 = "127.0.0.1:12345";
    let ip2 = "127.0.0.1:12346";
    let payload = "Test payload";

    // Start mock servers
    let server1 = start_mock_server(ip1).await?;
    let server2 = start_mock_server(ip2).await?;

    // Run the yalt application
    let opts = Opt {
        ips: vec![format!("{}:50", ip1), format!("{}:50", ip2)],
        rate,
        duration,
        payload: payload.to_string(),
    };
    run_yalt(opts, &db_path).await?;

    // Stop mock servers
    server1.abort();
    server2.abort();

    // Verify the results in the SQLite database
    let conn = Connection::open(&db_path)?;
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM metrics WHERE success = 1")?;
    let success_count: i64 = stmt.query_row([], |row| row.get(0))?;
    let expected_count = (rate as i64) * (duration as i64);
    // Allow a buffer of ±10%
    assert!(success_count >= expected_count * 90 / 100 && success_count <= expected_count * 110 / 100);

    Ok(())
}

async fn start_mock_server(addr: &str) -> Result<task::JoinHandle<()>, Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(addr).await?;
    let handle = task::spawn(async move {
        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buffer = [0; 1024];
            let _ = socket.read(&mut buffer).await;
            let _ = socket.write_all(&buffer).await;
        }
    });
    Ok(handle)
}
