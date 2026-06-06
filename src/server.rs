use rand::RngExt;// crates untuk random gen
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};


// Daftar Menu
#[derive(Debug, Clone)]
enum Item {
    Kopi,
    Susu,
    Roti,
}

impl Item {
    fn get_price(&self) -> u32 {
        match self {
            Item::Kopi
            Item::Susu
            Item::Roti
        }
    }
}

// Daftar Route yang valid
enum Route {
    Dashboard,
    Home,
    Add,
    Logout,
    NotFound,
}
// Data yang di bawa User
struct HttpRequest {
    route: Route,
    raw_request: String,
    user_id: String,
    is_new_usr: bool
}

impl HttpRequest {
    fn parse(buffer_str: &str) -> Self {
        let first_line = buffer_str.lines().next().unwrap_or("");
        let route = if first_line.starts_with("GET /dashboard"){
            Route::Dashboard
        }else if first_line.starts_with("GET /logout"){
            Route::Logout
        }else if first_line.starts_with("GET /tambah") {
            Route::Add
        }else if first_line.starts_with("GET /") || first_line.starts_with("GET /home"){
            Route::Home
        }else{
            Route::NotFound
        };
        let mut is_new_usr: bool = false;
        let user_id = if let Some(pos) = buffer_str.find("user_id=") {
            let start = pos + 8;
            let end = buffer_str[start..].find(";").unwrap_or_else(|| buffer_str[..start].len());
            buffer_str[start..start + end].to_string()
        }else{
            generate_random_cookie()
            is_new_usr = true
        }
        HttpRequest {
            route,
            raw_request: buffer_str.to_string(),
            user_id,
            is_new_usr
        }
    }
}

fn generate_random_cookie() -> String {
    let random_number: u32 = rand::rng().random();
    format!("user_{};", random_number)
}
pub fn handle_client(mut stream: TcpStream, inventory: Arc<Mutex<HashMap<String, Vec<Item>>>>) {
    let buffer = [0; 2024];// 2kb
    if let Ok(data_size) = stream.read(&mut buffer){
        let request_str = String::from_utf8_lossy(&buffer[..data_size]);
        let req = HttpRequest::parse(&request_str);
    }
    match req.route {
        Route::Home => //home
        Route::Dashboard => //Dashboard
        Route::Add => // adding process 
        Route::Logout => //Logout
        Route::NotFound => //NotFound
    }
}

fn adding_process(mut stream: TcpStream, request: &str, inventory: Arc<Mutex<HashMap<String, Vec<Item>>>>, user_id: &str) {
    {
        // kunci agar tidak berebut
        let mut map = match inventory.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("Mutex poisoned! Tapi kita recovery gemboknya.");
                poisoned.into_inner()
            }
        };
        // Proses parse String jadi emum Item
        let new_item = if request.contains("barang=kopi") {
            Some(Item::Kopi)
        }else if request.contains("barang=Susu"){
            Some(Item::Susu)
        }else if request.contains("barang=Roti"){
            Some(Item::Roti)
        }else{
            None
        };
        
        if Some(item) = new_item {
            inventory.push(item)// masukan enum bukan string 
        }
    }
    let response = "HTTP/1.1 303 See Other\r\nLocation: /dashboard\r\n\r\n";
    let _ = stream.write_all(response.as_bytes());
    drop(stream);
}

/*
// Fungsi Khusus Untuk web Home
fn home(mut stream: TcpStream, user_id: &str, is_new_usr: bool) {
    let status_line = "HTTP/1.1 200 OK";
    let html_content = include_str!("../home.html");
    let mut header = format!(
        "{}\r\nContent-Length: {}\r\nContent-Type: text/html",
        status_line,
        html_content.len()
    );
    if is_new_usr {
        header.push_str(&format!(
            "\r\nSet-Cookie: user_id={}; Path=/; HttpOnly",
            user_id
        ));
    }
    let response = format!("{}\r\n\r\n{}", header, html_content);
    let _ = stream.write_all(response.as_bytes());
}

fn dashboard(
    mut stream: TcpStream,
    inventory: Arc<Mutex<HashMap<String, Vec<String>>>>,
    user_id: &str,
) {
    println!("{:?}", inventory);
    let map = match inventory.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("Mutex poisoned! Tapi kita recovery gemboknya.");
            poisoned.into_inner()
        }
    };
    let status_line = "HTTP/1.1 200 OK";
    let html_content = include_str!("../dashboard.html");
    let mut item_html = String::new();
    let mut total = 0;
    // ambil inventory milik user
    if let Some(list_inventory) = map.get(user_id) {
        // cek isinya apakah kosong?
        if (list_inventory).is_empty() {
            item_html = String::from(
                "<div class='text-sm text-gray-500 text-center py-12'>Keranjang masih kosong nih... 🛍️</div>",
            );
        } else {
            item_html.push_str("<div class='space-y-2'>");
            for usr_item in list_inventory {
                item_html.push_str(&format!("<div class='flex justify-between items-center bg-gray-50 p-2 rounded-lg border border-gray-200'>
                        <span class='font-medium text-gray-700'>{}</span>
                    </div>",
                    usr_item));
                let usr_item = usr_item.to_lowercase();
                if usr_item.contains("kopi") {
                    total += 5000;
                } else if usr_item.contains("susu") {
                    total += 7000;
                } else if usr_item.contains("roti") {
                    total += 10000;
                }
            }
            item_html.push_str("</div>");
        }
    } else {
        // Kalo user belum nambahin item
        item_html = String::from(
            "<div class='text-sm text-gray-500 text-center py-12'>Keranjang masih kosong nih... 🛍️</div>",
        );
    }

    let final_html = html_content
        .replace("{{DAFTAR_BELANJAAN}}", &item_html)
        .replace(
            "<span class=\"text-green-600\">Rp 0</span>",
            &format!("<span class='text-green-600 font-bold'>Rp {}</span>", total),
        );
    let response = format!(
        "{}\r\nSet-Cookie: user_id={}; Path=/; HttpOnly\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
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

fn logout(
    mut stream: TcpStream,
    inventory: Arc<Mutex<HashMap<String, Vec<String>>>>,
    user_id: &str,
) {
    {
        let mut map = match inventory.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("Mutex poisoned! Tapi kita recovery gemboknya.");
                poisoned.into_inner()
            }
        };
        // hapus isi inventory biar nggak ada warisan
        map.remove(user_id);
    }
    let status_line = "HTTP/1.1 303 See Other";
    let response = format!(
        "{}\r\nLocation: /\r\nSet-Cookie: user_id=; Path=/; HttpOnly; Max-Age=0\r\n\r\n",
        status_line
    );
    let _ = stream.write_all(response.as_bytes());
}
*/