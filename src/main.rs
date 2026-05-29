use std::net::TcpListener;
use std::thread;
mod server;

/* Browser itu sifatnya stateless berarti sebenarnya dia tidak ingat koneksi lama tapi meminta request baru seperti mengulangi lagi tapi dengan jalur berbeda */
fn main() {
    match TcpListener::bind("127.0.0.1:8080") {
        Ok(listener) => {
            println!("Server is running...");
            // perulangan untuk menunggu Request incoming() artinya penjaga akan terus menunggu dan si simpan di stream berarti koneksi user
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        thread::spawn(move || {
                            server::handle_client(stream);
                            
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
