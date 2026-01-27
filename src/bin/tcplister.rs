use std::{io::Read, net::TcpListener, sync::mpsc, thread};

fn get_lines<R>(mut reader: R) -> mpsc::Receiver<String>
where
    R: Read + Send + 'static,
{
    let (sender, receiver) = mpsc::channel::<String>();

    thread::spawn(move || {
        let mut buf = [0; 8];
        let mut current_line = String::new();

        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let line = String::from_utf8_lossy(&buf[0..n]).to_string();
                    current_line.push_str(&line);

                    let mut parts: Vec<String> =
                        current_line.split('\n').map(String::from).collect();
                    current_line = parts.pop().unwrap_or_default();

                    for part in parts {
                        sender.send(part).unwrap();
                    }
                }
                Err(e) => {
                    eprint!("Error: {}", e);
                    break;
                }
            }
        }
        sender.send(current_line).unwrap();
    });

    receiver
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let receiver = get_lines(stream);
                for line in receiver {
                    println!("{}", line);
                }
            }
            Err(e) => {
                eprint!("Error: {}", e);
            }
        }
    }
}
