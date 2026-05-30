use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub fn handle_client(mut stream: TcpStream, inventory: Arc<Mutex<HashMap<String, Vec<String>>>>) {
    let mut buffer = [0; 2048]; // buffer untuk menampung Binner 
    match stream.read(&mut buffer) {
        Ok(data_size) => {
            // Terjemahkan Binner jadi utf8 lossy artinya kalau ada huruf aneh langsung buffer kita ambil sebesar data size
            let request = String::from_utf8_lossy(&buffer[..data_size]);
            /* browser modern kayak chrome atau browser bawaan hp itu super agresif! 1. request misterius (/favicon.ico): setiap kali kamu buka halaman web, browser itu gak cuma minta halaman html utama. */
            if request.starts_with("GET /favicon.ico") {
                return; // kalau browser cuma minta icon cuekin aja
            } else if request.starts_with("GET /dashboard") {
                // Halaman Dashboard
                dashboard(stream, inventory);
            } else if request.starts_with("GET /tambah") {
                // fungsi Tambahan Sekarang kita oper juga inventory-nya
                adding_process(stream, &request, inventory);
            } else if request.starts_with("GET / ") {
                // Halaman Home
                home(stream);
            } else {
                // Kalau Request tidak benar kirim Error 404
                error_404(stream);
            }
            println!("Koneksi masuk");
        }
        Err(e) => {
            println!("Gagal membaca data dari browser: {}", e);
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

fn dashboard(mut stream: TcpStream, inventory: Arc<Mutex<HashMap<String, Vec<String>>>>) {
    let map = inventory.lock().unwrap();
    let status_line = "HTTP/1.1 200 OK";
    // Ambil Source code
    let html_content = include_str!("../dashboard.html");

    // rakit teks html buat daftar barangnya
    let mut item_html = String::new();
    let mut total = 0;

    if map.is_empty() {
        item_html = String::from(
            "<div class='text-sm text-gray-500 text-center py-12'>Keranjang masih kosong nih... 🛍️</div>",
        );
    } else {
        // kalau ada isi, kita buat pembuka container-nya di sini
        item_html.push_str("<div class='space-y-2'>");

        // lakukan looping untuk merakit item
        for (user_id, list_inventory) in map.iter() {
            for item in list_inventory {
                item_html.push_str(&format!(
                    "<div class='flex justify-between items-center bg-gray-50 p-2 rounded-lg border border-gray-200'>
                        <span class='font-medium text-gray-700'>{}</span>
                    </div>",
                item
                ));

                // sekalian hitung total
                if item.contains("Kopi") {
                    total += 5000;
                } else if item.contains("Susu") {
                    total += 7000;
                } else if item.contains("Roti") {
                    total += 10000;
                }
            }
        }
    }
    // Pastikan penutup div container ini hanya berjalan di dalam blok ELSE
    item_html.push_str("</div>");

    // ganti place holder di html
    let mut final_html = html_content.replace("{{DAFTAR_BELANJAAN}}", &item_html);

    // ganti Text total Rp 0 dengan total harga asli
    final_html = final_html.replace(
        "<span class=\"text-green-600\">Rp 0</span>",
        &format!("<span class='text-green-600 font-bold'>Rp {}</span>", total),
    );

    // kirim hasilnya ke browser
    let response = format!(
        "{}\r\nContent-Length: {}\r\nContent-Type: text/html\r\n\r\n{}",
        status_line,
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
) {
    let user_id = "id123";
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
}
