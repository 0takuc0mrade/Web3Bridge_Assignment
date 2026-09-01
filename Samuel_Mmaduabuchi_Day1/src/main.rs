use std::io;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const ADDRESS: &str = "127.0.0.1:8080";
const MAX_REQUEST_SIZE: usize = 16 * 1024;

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind(ADDRESS).await?;
    println!("Restaurant server listening on http://{ADDRESS}");

    loop {
        let (stream, address) = listener.accept().await?;

        tokio::spawn(async move {
            if let Err(error) = handle_connection(stream).await {
                eprintln!("Could not handle request from {address}: {error}");
            }
        });
    }
}

async fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    let request = read_request(&mut stream).await?;
    let header_end = find_header_end(&request).unwrap_or(request.len());
    let headers = String::from_utf8_lossy(&request[..header_end]);
    let request_line = headers.lines().next().unwrap_or_default();

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let route = parts.next().unwrap_or_default();

    let (status, body) = match (method, route) {
        ("GET", "/menu") => (
            "200 OK",
            r#"{
  "foods": [
    "Jollof Rice",
    "Fried Rice",
    "Chicken",
    "Burger"
  ]
}"#,
        ),
        ("POST", "/order") => {
            if header_end + 4 <= request.len() {
                let order = String::from_utf8_lossy(&request[header_end + 4..]);
                println!("New order received:\n{}", order.trim());
            }

            (
                "200 OK",
                r#"{
  "message": "Order received successfully"
}"#,
            )
        }
        _ => (
            "404 Not Found",
            r#"{
  "error": "Route not found"
}"#,
        ),
    };

    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );

    stream.write_all(response.as_bytes()).await?;
    stream.shutdown().await
}

async fn read_request(stream: &mut TcpStream) -> io::Result<Vec<u8>> {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 1024];
    let mut expected_length = None;

    loop {
        let bytes_read = stream.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }

        request.extend_from_slice(&buffer[..bytes_read]);

        if request.len() > MAX_REQUEST_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "request is too large",
            ));
        }

        if let Some(header_end) = find_header_end(&request) {
            let total_length = *expected_length
                .get_or_insert_with(|| header_end + 4 + content_length(&request[..header_end]));

            if request.len() >= total_length {
                request.truncate(total_length);
                break;
            }
        }
    }

    Ok(request)
}

fn find_header_end(request: &[u8]) -> Option<usize> {
    request.windows(4).position(|window| window == b"\r\n\r\n")
}

fn content_length(headers: &[u8]) -> usize {
    String::from_utf8_lossy(headers)
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("Content-Length")
                .then(|| value.trim().parse().ok())
                .flatten()
        })
        .unwrap_or(0)
}
