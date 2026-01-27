use std::fs::File;
use std::io::{self, Result};
use std::net::TcpStream;

fn main() -> Result<()> {
    let mut file = File::open("messages.txt").expect("Failed to open file");

    let mut stream = TcpStream::connect("127.0.0.1:7878")?;
    println!("Connected to the server!");

    io::copy(&mut file, &mut stream)?;

    stream.shutdown(std::net::Shutdown::Write)?;

    println!("File sent successfully.");
    Ok(())
}
