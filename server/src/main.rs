use std::net::TcpListener;
use std::io::{ErrorKind, Read, Write};
use std::sync::{mpsc, Arc, Mutex };
use std::thread;
use std::collections::HashMap;

const LOCAL: &str = "192.168.0.21:6000";
const MSG_SIZE: usize = 32;

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(500));
}

fn main() {
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind");
    server.set_nonblocking(true).expect("Listener failed to initiate non-blocking state");

    let mut clients = vec![];
    let client_names = Arc::new(Mutex::new(HashMap::new()));
    let (tx, rx) = mpsc::channel::<String>();

    loop
    {
        if let Ok((mut socket, addr)) = server.accept()
        {
            println!("Client connected: {}", addr);

            let client_names = Arc::clone(&client_names);
            let tx = tx.clone();
            clients.push(socket.try_clone().expect("Failed to clone client"));

            thread::spawn(move || loop
            {
                let mut buff = vec![0; MSG_SIZE];

                match socket.read_exact(&mut buff)
                {
                   Ok(_) => 
                   {
                       let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                       let msg = String::from_utf8(msg).expect("Invalid utf8 message");

                       if msg.starts_with(":reg")
                       {
                        let name = msg.replace(":reg::", "");
                        println!("Registered: {}", name);
                        let mut names = client_names.lock().unwrap();
                        names.insert(addr.clone(), name.clone());
                        tx.send(format!("{} joined the chat", name)).expect("failed to send msg to rx");
                       }
                       else
                       {
                        println!("{}: {:?}", addr, msg);
                        if !client_names.lock().unwrap().contains_key(&addr)
                        {
                            tx.send(format!("[Unregistered]: {}", msg)).expect("failed to msg to rx");
                        }
                        else
                        {
                            tx.send(format!("[{}]: {}", client_names.lock().unwrap().get(&addr).unwrap(), msg )).expect("failed to send msg to rx");
                        }
                        
                       }
                       
                   }, 

                   Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                   Err(_) =>
                   {
                       println!("Closing connection with: {}", addr);
                       if client_names.lock().unwrap().contains_key(&addr)
                       {
                           tx.send(format!("{} left the chat", client_names.lock().unwrap().get(&addr).unwrap())).expect("failed to send msg to rx");
                       }
                       else
                       {
                           tx.send("Unregistered disconnected".to_string()).expect("failed to send msg to rx");
                       }

                       break;
                   }
                }

                sleep();

            });
        }

        if let Ok(msg) = rx.try_recv()
        {
            clients = clients.into_iter().filter_map
            (
                |mut client|
                {
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);
                    client.write_all(&buff).map(|_| client).ok()
                }
            ).collect::<Vec<_>>();
        }

        sleep();
    }
}
