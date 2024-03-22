use tokio::{ io::{ AsyncBufReadExt, AsyncWriteExt, BufReader }, net::TcpListener, sync::broadcast };

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.expect("Server failed");

    let (sender, _receiver) = broadcast::channel(3);

    loop {
        let (mut socket, add) = listener.accept().await.unwrap();

        let sender = sender.clone();

        let mut receiver = sender.subscribe();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();

            let mut reader = BufReader::new(reader);

            let mut line = String::new();

            loop {
                tokio::select! {
                    result = reader.read_line(&mut line)=>{

                        if result.unwrap() == 0 {
                            return;
                        }

                        sender.send((line.clone(),add)).unwrap();

                        line.clear();
        
                    }

                    result =  receiver.recv()=>{
                        let (message,other_add) = result.unwrap();
                        if add != other_add{
                            writer.write_all(message.as_bytes()).await.expect("Error");
                        }
                    }
                }
            }
        });
    }
}
