use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use rand::Rng;// library bawaan untuk random integer

fn generate_random_cookie() -> String {
    let random_number: u32 = rand::thread_rng().gen();
    format!("user_{}", random_number)
}

pub fn handle_client(mut stream: TcpStream, inventory: Arc<Mutex<HashMap<String, Vec<String>>>>) {
    let mut buffer = [0; 2048]; // buffer untuk menampung Binner 
    if let Ok(data_size) = stream.read(&mut buffer) {
        let request = String::from_utf8_lossy(&buffer[..data_size]);
        let user_id = if let Some(pos) = request.find("user_id="){
            let start = pos + 8;
            let end = request[start..].find(|c| c == ';' || c == '\r').unwrap_or(request[start..].len());
            request[start..start + end].to_string()
        }else{
            generate_random_cookie()
        };
        if request.starts_with("GET /dashboard"){
            // dashboard
            dashboard(stream, inventory, &user_id);
        } else if request.starts_with("GET /tambah"){
            // fungsi tambah
            adding_process(stream, &request, inventory, &user_id);
        }else {
            home(stream);
        }
    }
}

// Fungsi Khusus Untuk web Home
fn home(mut stream: TcpStream) {
    let status_line = "HTTP/1.1 200 OK";
    let html_content = include_str!("../home.html");
    let response = format!(
        "{}\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
        status_line,
        html_content.len(),
        html_content
    );
    let _ = stream.write_all(response.as_bytes());
}

fn dashboard(mut stream: TcpStream, inventory: Arc<Mutex<HashMap<String, Vec<String>>>>,  user_id: &str) {
    let map = inventory.lock().unwrap();
    let status_line = "HTTP/1.1 200 OK";
    let html_content = include_str!("../dashboard.html");
    let mut item_html = String::new();
    let mut total = 0;
    // ambil inventory milik user
    if let Some(list_inventory) = map.get(user_id) {
        // cek isinya apakah kosong?
        if (list_inventory).is_empty() {
            item_html = String::from("<div class='text-sm text-gray-500 text-center py-12'>Keranjang masih kosong nih... 🛍️</div>");
            
        }else {
            item_html.push_str("<div class='space-y-2'>");
            for usr_item in list_inventory {
                item_html.push_str(&format!("<div class='flex justify-between items-center bg-gray-50 p-2 rounded-lg border border-gray-200'>
                        <span class='font-medium text-gray-700'>{}</span>
                    </div>",
                    usr_item));
                    let usr_item = usr_item.to_lowercase();
                    if usr_item.contains("kopi"){total += 5000;}
                    else if usr_item.contains("susu"){total += 7000;}
                    else if usr_item.contains("roti"){total += 10000;}
            }
            item_html.push_str("</div>");
        }
    }else{
        // Kalo user belum nambahin item
        item_html = String::from("<div class='text-sm text-gray-500 text-center py-12'>Keranjang masih kosong nih... 🛍️</div>");
    }
    let final_html = html_content.replace("{{DAFTAR_BELANJAAN}}", &item_html)
    .replace("<span class=\"text-green-600\">Rp 0</span>", &format!("<span class='text-green-600 font-bold'>Rp {}</span>", total));
    let response = format!("{}\r\nSet-Cookie: user_id={}; Path=/; HttpOnly\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
        status_line,
        user_id,
        final_html.len(),
        final_html
    );
    let _ = stream.write_all(response.as_bytes());
}

// Fungsi kalau Halaman tidak ditemukan
fn error_404(mut stream: TcpStream) {
    let status_line = "HTTP/1.1 404 NOT FOUND";
    let html_content =
        "<h1> Oops! Halaman tidak ditemukan 😥</h1><a href='/'>Kembali Ke Jalan Setan</a>";
    let response = format!(
        "{}\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
        status_line,
        html_content.len(),
        html_content
    );
    let _ = stream.write_all(response.as_bytes());
}

// Sekarang fungsi ini menerima inventory dan beneran bisa menyimpan data
fn adding_process(
    mut stream: TcpStream,
    request: &str,
    inventory: Arc<Mutex<HashMap<String, Vec<String>>>>,
    user_id: &str
) {
    // kunci agar tidak berebut
    let mut map = inventory.lock().unwrap();
    //
    let inventory = map.entry(user_id.to_string()).or_insert(Vec::new()); // ambil atau berikan vektor kosong
    if request.contains("barang=kopi") {
        inventory.push(String::from("Kopi Hangat"));
    } else if request.contains("barang=susu") {
        inventory.push(String::from("Susu Segar"));
    } else if request.contains("barang=roti") {
        inventory.push(String::from("Roti Bakar"));
    }

    let response = "HTTP/1.1 303 See Other\r\nLocation: /dashboard\r\n\r\n";
    let _ = stream.write_all(response.as_bytes());
    drop(stream);
}
