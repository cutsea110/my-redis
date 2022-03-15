use mini_redis::{Connection, Frame};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    // リスナーをこのアドレスにバインドする
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        // タプルの2つ目の要素は、新しいコネクションのIPとポートの情報を含んでいる
        let (socket, _) = listener.accept().await.unwrap();
        // それぞれのインバウンドソケットに対して新しいタスクを spawn する
        // ソケットは新しいタスクに move され、そこで処理される
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(socket: TcpStream) {
    use mini_redis::Command::{self, Get, Set};
    use std::collections::HashMap;

    // データを蓄えるため HashMap を使う
    let mut db = HashMap::new();

    // mini-redis が提供するコネクションによって、ソケットからくるフレームをパースする
    let mut connection = Connection::new(socket);

    // コネクションからコマンドを受け取るため read_frame を使う
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                // Vec<u8> として保存する
                db.insert(cmd.key().to_string(), cmd.value().to_vec());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                if let Some(value) = db.get(cmd.key()) {
                    // Frame::Bulk はデータが Bytes 型であることを期待する
                    // ここでは &Vec<u8> から Bytes に変換する
                    Frame::Bulk(value.clone().into())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd),
        };

        // クライアントへのレスポンスを書きこむ
        connection.write_frame(&response).await.unwrap();
    }
}
