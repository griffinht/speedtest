use std::io::{BufRead, Write};

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
    let stream = stream?;
    let address = stream.peer_addr().unwrap().ip();
    println!("{}", address);
    let mut reader = std::io::BufReader::new(stream.try_clone()?);
    let mut writer = std::io::BufWriter::new(stream.try_clone()?);
    let mut protocol = Vec::new();
    reader.read_until(b' ', &mut protocol)?;
    let protocol = match &protocol[0..protocol.len() - 1] {
        b"GET" => "GET",
        b"POST" => "POST",
        _ => {
            writer.write(b"HTTP/1.1 405 Method Not Allowed\r\n\r\n")?;
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("method not allowed: {} (must be GET or POST)", from_utf8(&protocol)?)));
        }
    };
    let mut target = Vec::new();
    reader.read_until(b' ', &mut target)?;
    let target = from_utf8(&target[0..target.len() - 1])?;
}

fn from_utf8(bytes: &[u8]) -> std::io::Result<&str> {
    match std::str::from_utf8(bytes) {
        Ok(str) => Ok(str),
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("invalid utf8: {}", e)))
    }
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