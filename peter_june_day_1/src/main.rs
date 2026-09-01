use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    println!("Server running on http://127.0.0.1:8080");

    loop {
        let (mut socket, address) = listener
            .accept()
            .await
            .unwrap();

        println!("New connection from {address}");

        tokio::spawn(async move {
            handle_connection(socket).await;
        });
    }
}



async fn handle_connection(mut socket: tokio::net::TcpStream) {
    let mut buffer = [0; 1024];

    let bytes_read = socket
        .read(&mut buffer)
        .await
        .unwrap();

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);

    println!("{}", request);

    let response =
        "HTTP/1.1 200 OK\r\n\
         Content-Length: 13\r\n\
         Connection: close\r\n\
         \r\n\
         Hello, world!";

    socket
        .write_all(response.as_bytes())
        .await
        .unwrap();
}