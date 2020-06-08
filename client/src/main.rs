use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;

const LOCAL: &str = "192.168.0.42:6000";
const MSG_SIZE: usize = 32;

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(500));
}

fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    client.set_nonblocking(true).expect("failed to initialise non-blocking");

    let (tx,rx) = mpsc::channel::<String>();

    thread::spawn(move || loop
    {
        let mut buff = vec![0; MSG_SIZE];

        match client.read_exact(&mut buff)
        {
            Ok(_) =>
            {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                println!("{}", msg);
            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => 
            {
               println!("Connection with server lost."); 
               break;
            }
        }

        match rx.try_recv()
        {
            Ok(mgs) =>
            {
               let mut buff = mgs.clone().into_bytes(); 
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("Writing to socket failed");
                // println!("message sent {:?}", mgs);
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break

        }

        sleep();
    });

    loop
    {
        println!("Enter your name:");
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("reading from stdin failed");
        if buff.trim().is_empty()
        {
            println!("You cannot register as an empty name");
        }
        else
        {
            let msg = format!(":reg::{}", buff.trim().to_string());
            if tx.send(msg).is_ok() {break}
            println!("\n");
        }
    }


    println!("Write a message: ");
    loop
    {
       let mut buff = String::new();
       io::stdin().read_line(&mut buff).expect("reading from stdin failed");
       let msg = buff.trim().to_string();
       if msg  == ":quit" || tx.send(msg).is_err() {break}
    }
    println!("Thank you for using my chat client");

}
