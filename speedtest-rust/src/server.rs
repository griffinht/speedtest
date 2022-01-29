use std::io::{Read, Write};

pub fn listen<A: std::net::ToSocketAddrs>(address: A) -> std::io::Result<()> {
    let listener = std::net::TcpListener::bind(address)?;
    eprintln!("listening on {}", listener.local_addr().unwrap());
    for stream in listener.incoming() {
        match handle(stream) {
            Ok(_) => {}
            Err(e) => { eprintln!("{}", e); }
        }
    }
    Ok(())
}

fn handle(stream: std::io::Result<std::net::TcpStream>) -> std::io::Result<()> {
    let mut stream = stream?;
    let address = stream.peer_addr().unwrap().ip();
    println!("{}", address);
    let protocol = &mut [0u8; 1];
    stream.read_exact(protocol)?;
    match protocol[0] {
        b'G' => { stream.write(&get_http(address)) } // GET
        b'P' => { stream.write(&get_http(address)) } // POST
        _ => {
            eprintln!("bad protocol: must be {} or HTTP GET or POST", env!("CARGO_PKG_NAME"));
            stream.write(b"HTTP/1.1 405 Method Not Allowed\r\n\r\n")
        }
    }?;
    Ok(())
}

fn get_http(address: std::net::IpAddr) -> Vec<u8> {
    let mut headers: Vec<u8> = Vec::new();
    let mut body: Vec<u8> = Vec::new();

    body.extend_from_slice(match address {
        std::net::IpAddr::V4(ip) => {
            ip.to_string()
        },
        std::net::IpAddr::V6(ip) => {
            ip.to_string()
        }
    }.as_bytes());

    headers.extend_from_slice(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain;\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: ");
    headers.extend_from_slice(body.len().to_string().as_bytes());
    headers.extend_from_slice(b"\r\n\r\n");

    let mut response = headers;
    response.extend_from_slice(&body);

    return response;
}