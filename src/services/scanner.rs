use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::time::timeout;

const COMMON_PORTS: &[(u16, &str)] = &[
    (21, "FTP"),
    (22, "SSH"),
    (23, "Telnet"),
    (25, "SMTP"),
    (53, "DNS"),
    (80, "HTTP"),
    (110, "POP3"),
    (143, "IMAP"),
    (443, "HTTPS"),
    (445, "SMB"),
    (993, "IMAPS"),
    (995, "POP3S"),
    (1433, "MSSQL"),
    (1521, "Oracle"),
    (2049, "NFS"),
    (3306, "MySQL"),
    (3389, "RDP"),
    (5432, "PostgreSQL"),
    (5900, "VNC"),
    (6379, "Redis"),
    (8080, "HTTP-Alt"),
    (8443, "HTTPS-Alt"),
    (27017, "MongoDB"),
];

#[derive(Debug, Clone)]
pub struct OpenPort {
    pub port: u16,
    pub service: &'static str,
}

async fn try_connect(addr: &str, dur: Duration) -> bool {
    timeout(dur, TcpStream::connect(addr)).await.is_ok()
}

pub async fn scan_common_ports(
    ip: IpAddr,
    timeout_ms: u64,
    concurrency: usize,
) -> Vec<OpenPort> {
    let dur = Duration::from_millis(timeout_ms);
    let sem = Arc::new(Semaphore::new(concurrency));
    let results: Vec<OpenPort> = Vec::with_capacity(COMMON_PORTS.len());
    let shared = Arc::new(parking_lot::Mutex::new(results));

    let mut handles = Vec::with_capacity(COMMON_PORTS.len());
    for &(port, service) in COMMON_PORTS {
        let permit = sem.clone().acquire_owned().await.unwrap();
        let addr = format!("{}:{}", ip, port);
        let sh = shared.clone();

        handles.push(tokio::spawn(async move {
            if try_connect(&addr, dur).await {
                sh.lock().push(OpenPort { port, service });
            }
            drop(permit);
        }));
    }

    for h in handles {
        let _ = h.await;
    }

    Arc::try_unwrap(shared).unwrap().into_inner()
}

pub async fn scan_port_range(
    ip: IpAddr,
    start: u16,
    end: u16,
    timeout_ms: u64,
    concurrency: usize,
) -> Vec<OpenPort> {
    let dur = Duration::from_millis(timeout_ms);
    let sem = Arc::new(Semaphore::new(concurrency));
    let count = (end - start + 1) as usize;
    let results: Vec<OpenPort> = Vec::with_capacity(count);
    let shared = Arc::new(parking_lot::Mutex::new(results));

    let mut handles = Vec::with_capacity(count);
    for port in start..=end {
        let _permit = sem.clone().acquire_owned().await.unwrap();
        let addr = format!("{}:{}", ip, port);
        let sh = shared.clone();

        handles.push(tokio::spawn(async move {
            if try_connect(&addr, dur).await {
                sh.lock().push(OpenPort {
                    port,
                    service: guess_service(port),
                });
            }
            drop(_permit);
        }));
    }

    for h in handles {
        let _ = h.await;
    }

    Arc::try_unwrap(shared).unwrap().into_inner()
}

fn guess_service(port: u16) -> &'static str {
    for &(p, name) in COMMON_PORTS {
        if p == port {
            return name;
        }
    }
    "Unknown"
}
