extern crate csv;
extern crate rustc_serialize;
#[macro_use]
extern crate hyper;
extern crate rayon;

use std::io;
use std::io::Write;
use std::fs::OpenOptions;
use std::thread;
use std::sync::Arc;
use csv::Reader;
use hyper::{Url, Client};
use hyper::client::response::Response;
use hyper::header::{Headers, UserAgent};
use rayon::prelude::*;

header! { (Accept, "Accept") => [String] }
header! { (AcceptLanguage, "AcceptLanguage") => [String] }

#[derive(RustcDecodable)]
struct Bookmark {
    title: String,
    url: String,
    tags: String,
    description: String,
    comments: String,
    annotations: String,
    created_at: String,
}

const BOOKMARKS_PATH: &'static str = "../3070477_csv_2017_01_08_56075.csv";
const ERROR_LOG_PATH: &'static str = "./error.log";

fn log_error(message: &String) {
    let output = message.to_string() + "\n";
    let output_bytes = output.as_bytes();
    match io::stdout().write_all(&output_bytes) {
        Ok(_) => (),
        Err(_) => (),
    }
    let error_log_result = OpenOptions::new()
        .create(true)
        .append(true)
        .open(ERROR_LOG_PATH);
    if let Ok(mut error_log) = error_log_result {
        match error_log.write_all(&output_bytes) {
            Ok(_) => (),
            Err(_) => (),
        }
    }
}

fn check_bookmark(bookmark: &Bookmark) {
    // println!("Start {}", bookmark.url);

    let url_result = Url::parse(&bookmark.url);
    if let Err(err) = url_result {
        log_error(&format!("Could not parse {}: {}", bookmark.url, err));
        return;
    }
    let url = url_result.unwrap();

    let mut headers = Headers::new();
    headers.set(UserAgent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_4) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/57.0.2987.133 Safari/537.36".to_string()));
    headers.set(Accept("text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8".to_string()));
    headers.set(AcceptLanguage("en,de;q=0.8".to_string()));

    let result = Client::new()
        .head(url)
        .headers(headers)
        .send();
    match result {
        Ok(response) => {
            let status = response.status;
            if status.is_success() {
                println!("{} {}", bookmark.url, response.status);
            } else {
                log_error(&format!("{} {}", bookmark.url, status));
            }
        }
        Err(err) => {
            log_error(&format!("{} {:?}", bookmark.url, err));
        }
    };
}

fn main() {
    Reader::from_file(BOOKMARKS_PATH)
        .unwrap()
        .decode()
        .collect::<csv::Result<Vec<Bookmark>>>()
        .unwrap()
        .par_iter()
        .map(|bookmark| {
            check_bookmark(bookmark)
        })
        .collect::<Vec<_>>();
}
