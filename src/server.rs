use rand::RngExt; // benerin traits random
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum Item {
    Kopi,
    Susu,
    Roti,
}

impl Item {
    fn get_price(&self) -> u32 {
        match self {
            Item::Kopi => 5000,
            Item::Susu => 7000,
            Item::Roti => 10000,
        }
    }
}

enum Route {
    Dashboard,
    Home,
    Add,
    Logout,
    NotFound,
}

struct HttpRequest {
    route: Route,
    raw_request: String,
    user_id: String,
    is_new_usr: bool,
}

impl HttpRequest {
    fn parse(buffer_str: &str) -> Self {
        let first_line = buffer_str.lines().next().unwrap_or("");
        let route = if first_line.starts_with("GET /dashboard") {
            Route::Dashboard
        } else if first_line.starts_with("GET /logout") {
            Route::Logout
        } else if first_line.starts_with("GET /tambah") {
            Route::Add
        } else if first_line.starts_with("GET /") || first_line.starts_with("GET /home") {
            Route::Home
        } else {
            Route::NotFound
        };
        
        let mut is_new_usr = false;
        let mut user_id = String::new();
        
        if let Some(pos) = buffer_str.find("user_id=") {
            let start = pos + 8;
            let end = buffer_str[start..].find(";").unwrap_or_else(|| buffer_str[start..].len());
            user_id = buffer_str[start..start + end].to_string();
        } else {
            user_id = generate_random_cookie();
            is_new_usr = true;
        }
        
        HttpRequest {
            route,
            raw_request: buffer_str.to_string(),
            user_id,
            is_new_usr,
        }
    }
}

fn generate_random_cookie() -> String {
    let random_number: u32 = rand::rng().random();
    format!("user_{}", random_number)
}

pub fn handle_client(mut stream: TcpStream, inventory: Arc<Mutex<HashMap<String, Vec<Item>>>>) {
    let mut buffer = [0; 2024];
    if let Ok(data_size) = stream.read(&mut buffer) {
        let request_str = String::from_utf8_lossy(&buffer[..data_size]);
        let req = HttpRequest::parse(&request_str);
        
        // match dipindah ke dalem biar dapet variabel req-nya, yank!
        match req.route {
            Route::Home => home(stream, &req.user_id, req.is_new_usr),
            Route::Dashboard => dashboard(stream, inventory, &req.user_id),
            Route::Add => adding_process(stream, &req.raw_request, inventory, &req.user_id),
            Route::Logout => logout(stream, inventory, &req.user_id),
            Route::NotFound => error_404(stream),
        }
    }
}

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

fn dashboard(mut stream: TcpStream, inventory: Arc<Mutex<HashMap<String, Vec<Item>>>>, user_id: &str) {
    let map = match inventory.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner()
    };
    let status_line = "HTTP/1.1 200 OK";
    let html_content = include_str!("../dashboard.html");
    let mut item_html = String::new();
    let mut total = 0;
    
    if let Some(items) = map.get(user_id) {
        if items.is_empty() {
            item_html = String::from("<div class='text-sm text-gray-500 text-center py-12'>Keranjang masih kosong nih... 🛍️</div>");
        } else {
            item_html.push_str("<div class='space-y-2'>");
            for item in items {
                total += item.get_price();
                let item_name = match item {
                    Item::Kopi => "Kopi Hangat",
                    Item::Susu => "Susu Hangat",
                    Item::Roti => "Roti",
                };
                item_html.push_str(&format!("<li>{} - Rp{}</li>", item_name, item.get_price()));
            }
            item_html.push_str("</div>");
        }
    } else {
        item_html = String::from("<div class='text-sm text-gray-500 text-center py-12'>Keranjang masih kosong nih... 🛍️</div>");
    }

    let final_html = html_content
        .replace("{{DAFTAR_BELANJAAN}}", &item_html)
        .replace("<span class=\"text-green-600\">Rp 0</span>", &format!("<span class='text-green-600 font-bold'>Rp {}</span>", total));
        
    let response = format!(
        "{}\r\nSet-Cookie: user_id={}; Path=/; HttpOnly\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
        status_line, user_id, final_html.len(), final_html
    );
    let _ = stream.write_all(response.as_bytes());
}

fn adding_process(mut stream: TcpStream, request: &str, inventory: Arc<Mutex<HashMap<String, Vec<Item>>>>, user_id: &str) {
    {
        let mut map = match inventory.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner()
        };
        // ambil atau bikin vector baru buat user ini
        let user_cart = map.entry(user_id.to_string()).or_insert(Vec::new());
        
        if request.contains("barang=kopi") {
            user_cart.push(Item::Kopi);
        } else if request.contains("barang=susu") { // samain lowercase-nya yank
            user_cart.push(Item::Susu);
        } else if request.contains("barang=roti") {
            user_cart.push(Item::Roti);
        }
    }
    let response = "HTTP/1.1 303 See Other\r\nLocation: /dashboard\r\n\r\n";
    let _ = stream.write_all(response.as_bytes());
}

fn logout(mut stream: TcpStream, inventory: Arc<Mutex<HashMap<String, Vec<Item>>>>, user_id: &str) {
    {
        let mut map = match inventory.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner()
        };
        map.remove(user_id);
    }
    let status_line = "HTTP/1.1 303 See Other";
    let response = format!(
        "{}\r\nLocation: /\r\nSet-Cookie: user_id=; Path=/; HttpOnly; Max-Age=0\r\n\r\n",
        status_line
    );
    let _ = stream.write_all(response.as_bytes());
}

fn error_404(mut stream: TcpStream) {
    let status_line = "HTTP/1.1 404 NOT FOUND";
    let html_content = "<h1> Oops! Halaman tidak ditemukan 😥</h1><a href='/'>Kembali Ke Jalan Setan</a>";
    let response = format!(
        "{}\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
        status_line, html_content.len(), html_content
    );
    let _ = stream.write_all(response.as_bytes());
}
