use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tokio::time::{sleep, Duration, Instant};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use rusqlite::{params, Connection, Result};
use rand::Rng;
use std::path::Path;
use structopt::StructOpt;

pub struct TokenBucket {
    rate: f64, // Tokens added per second
    capacity: f64, // Maximum bucket capacity
    tokens: f64, // Current number of tokens
    last_update: Instant, // Use Instant for more precise time measurement
}

impl TokenBucket {
    pub fn new(rate: f64, capacity: f64) -> Self {
        TokenBucket {
            rate,
            capacity,
            tokens: capacity,
            last_update: Instant::now(),
        }
    }

    pub fn add_tokens(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        let added_tokens = elapsed * self.rate;
        self.tokens = f64::min(self.capacity, self.tokens + added_tokens);
        self.last_update = now;
    }

    pub fn allow_request(&mut self) -> bool {
        self.add_tokens();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

#[derive(StructOpt, Debug)]
pub struct Opt {
    /// IP addresses to send payloads to with biases, e.g. "127.0.0.1:12345:50"
    pub ips: Vec<String>,

    /// Requests per second
    pub rate: f64,

    /// Duration in seconds
    pub duration: u64,

    /// Payload to send
    pub payload: String,
}

pub async fn run_yalt(opt: Opt, db_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let bucket = Arc::new(Mutex::new(TokenBucket::new(opt.rate, opt.rate)));
    let start_time = Instant::now();
    let mut sent_requests = 0;

    let conn = Arc::new(Mutex::new(Connection::open(db_path)?));
    conn.lock().unwrap().execute(
        "CREATE TABLE IF NOT EXISTS metrics (
            id INTEGER PRIMARY KEY,
            timestamp INTEGER NOT NULL,
            ip TEXT NOT NULL,
            payload TEXT NOT NULL,
            success BOOLEAN NOT NULL
        )",
        [],
    )?;

    let targets: Arc<Vec<(String, u16, u32)>> = Arc::new(opt.ips.iter()
        .map(|ip| {
            let parts: Vec<&str> = ip.split(':').collect();
            let ip = parts[0].to_string();
            let port: u16 = parts[1].parse().expect("Invalid port");
            let bias: u32 = parts[2].parse().expect("Invalid bias");
            (ip, port, bias)
        })
        .collect());

    let total_bias: u32 = targets.iter().map(|(_, _, bias)| *bias).sum();

    while Instant::now().duration_since(start_time) < Duration::from_secs(opt.duration) {
        {
            let mut bucket = bucket.lock().unwrap();
            if bucket.allow_request() {
                sent_requests += 1;
                let payload = opt.payload.clone();
                let conn = Arc::clone(&conn);
                let targets = Arc::clone(&targets);
                tokio::spawn(async move {
                    let target = select_target(&targets, total_bias);
                    let success = send_payload(target.0.clone(), target.1, payload.clone()).await;
                    store_metric(&conn, target.0.clone(), payload.clone(), success).unwrap();
                });
            }
        }
        sleep(Duration::from_millis(1)).await;
    }

    println!("Total requests sent: {}", sent_requests);
    println!("Average RPS: {:.2}", sent_requests as f64 / opt.duration as f64);
    Ok(())
}

pub fn generate_db_filename(opt: &Opt) -> String {
    let ip_info: Vec<String> = opt.ips.iter().map(|ip| ip.replace(':', "_")).collect();
    let ip_info_str = ip_info.join("_");
    format!("{}_rate{}_dur{}.db", ip_info_str, opt.rate, opt.duration)
}

fn select_target(targets: &Vec<(String, u16, u32)>, total_bias: u32) -> &(String, u16, u32) {
    let mut rng = rand::thread_rng();
    let mut rnd = rng.gen_range(0..total_bias);
    for target in targets {
        if rnd < target.2 {
            return target;
        }
        rnd -= target.2;
    }
    &targets[0]
}

async fn send_payload(ip: String, port: u16, payload: String) -> bool {
    match TcpStream::connect((ip.as_str(), port)).await {
        Ok(mut stream) => {
            if let Err(e) = stream.write_all(payload.as_bytes()).await {
                eprintln!("Failed to send payload to {}: {}: {}", ip, port, e);
                return false;
            }
            true
        }
        Err(e) => {
            eprintln!("Failed to connect to {}: {}: {}", ip, port, e);
            false
        }
    }
}

fn store_metric(conn: &Arc<Mutex<Connection>>, ip: String, payload: String, success: bool) -> Result<()> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
    let conn = conn.lock().unwrap();
    conn.execute(
        "INSERT INTO metrics (timestamp, ip, payload, success) VALUES (?1, ?2, ?3, ?4)",
        params![timestamp, ip, payload, success],
    )?;
    Ok(())
}
