use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind to 127.0.0.1:8080");

    println!("Restaurant server listening on 127.0.0.1:8080");

    loop {
        let (stream, addr) = listener.accept().await.expect("Failed to accept connection");

        println!("New connection from {}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
}

async fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await?;

    let request = String::from_utf8_lossy(&buffer[..n]);

    let response = if request.starts_with("GET /menu") {
        get_menu()
    } else if request.starts_with("POST /order") {
        handle_order(&request).await
    } else {
        not_found()
    };

    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;

    Ok(())
}

fn get_menu() -> String {
    let body = r#"{"foods":["Jollof Rice","Fried Rice","Chicken","Burger"]}"#;

    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    )
}

async fn handle_order(request: &str) -> String {
    // Extract the body from the request
    if let Some(body_start) = request.find("\r\n\r\n") {
        let body = &request[body_start + 4..];

        // Try to parse food and quantity from the JSON-like body
        if let (Some(food_start), Some(food_end)) = (
            body.find("\"food\":\""),
            body[body.find("\"food\":\"").unwrap_or(0) + 8..].find("\""),
        ) {
            let food = &body[food_start + 8..food_start + 8 + food_end];
            if let Some(qty_start) = body.find("\"quantity\":") {
                let qty_part = &body[qty_start + 11..];
                if let Some(qty_end) = qty_part.find(|c: char| !c.is_numeric()) {
                    let quantity = &qty_part[..qty_end];
                    println!("New order received:");
                    println!("Food: {}", food);
                    println!("Quantity: {}", quantity);
                }
            }
        }
    }

    let body = r#"{"message":"Order received successfully"}"#;

    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    )
}

fn not_found() -> String {
    let body = r#"{"error":"Route not found"}"#;

    format!(
        "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    )
}
