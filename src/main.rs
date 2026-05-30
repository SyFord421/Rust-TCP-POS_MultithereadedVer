use std::collections::HashMap;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

mod server;
/* Browser itu sifatnya stateless berarti sebenarnya dia tidak ingat koneksi lama tapi meminta request baru seperti mengulangi lagi tapi dengan jalur berbeda */
fn main() {
    // Ubah baris ini di main.rs:
    let inventory = Arc::new(Mutex::new(HashMap::<String, Vec<String>>::new()));

    match TcpListener::bind("127.0.0.1:8080") {
        Ok(listener) => {
            println!("Server is running...");
            // perulangan untuk menunggu Request incoming() artinya penjaga akan terus menunggu dan si simpan di stream berarti koneksi user
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let invent_clone = Arc::clone(&inventory);
                        thread::spawn(move || {
                            server::handle_client(stream, invent_clone);
                        });
                    }
                    Err(e) => println!("Gagal menerima koneksi karena: {}", e),
                }
            }
        }
        Err(e) => {
            println!("Waduh, server gagal dinyalain karena: {}", e);
        }
    }
}
