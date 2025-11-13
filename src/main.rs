use std::{fs::File, io::Read, sync::mpsc, thread};

fn stream_lines<R>(mut reader: R) -> mpsc::Receiver<String>
where R: Read + Send + 'static {
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

                    let mut parts: Vec<String> = current_line.split('\n').map(String::from).collect();
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
    let file = File::open("messages.txt").expect("Failed to open file");
    let receiver = stream_lines(file);
    for line in receiver {
        println!("{}", line);
    }
}