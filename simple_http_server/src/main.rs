extern crate hyper;
extern crate env_logger;

use std::io::copy;
use std::env;
use std::fs::File;
use std::path::PathBuf;

use hyper::{Get, Post};
//use hyper::net::Openssl;
use hyper::header::ContentType;
use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri::AbsolutePath;
use hyper::mime::{Mime, TopLevel, SubLevel};

macro_rules! try_return(
    ($e:expr) => {{
        match $e {
            Ok(v) => v,
            Err(e) => { println!("Error: {}", e); return; }
        }
    }}
);

fn is_under_folder(ancestor: &PathBuf, target: &PathBuf) -> bool {
  if ancestor.eq(target) {
      return true;
  }
  let mut parent = target.parent();
  loop {
      match parent {
          None => {
              return false;
          },
          _ => {
              let real_parent = parent.unwrap();
              if real_parent.eq(ancestor) {
                  return true;
              } else {
                parent = real_parent.parent();
              }
          }
      }
  }
}

fn show_file(path: &String, mut res: Response) {
    let cwd = env::current_dir().unwrap();
    let relative_path = &path[1..];
    let mut target = cwd.join(relative_path);

    if !is_under_folder(&cwd, &target) || !target.exists() {
        target = cwd;
    }
    println!("Get {} target: {}", relative_path, target.display());
    if target.is_file() {
        let octet_type = String::from("octet-stream");
        res.headers_mut().set(ContentType(Mime(TopLevel::Application,
                SubLevel::Ext(octet_type), vec![])));
        let mut res = try_return!(res.start());
        let mut file_reader = try_return!(File::open(target));
        try_return!(copy(&mut file_reader, &mut res));
    } else if target.is_dir() {
        let mut content = String::from("<html><head></head><body><ul>");
        let entries = target.read_dir().unwrap();
        for entry in entries {
            let entry = entry.unwrap();
            let entry_name = entry.file_name().to_string_lossy().into_owned();
            let mut is_dir = "/";
            let entry_file_type = entry.file_type().unwrap();
            if entry_file_type.is_file() {
                is_dir = "";
            }
            content.push_str(&format!("<li><a href=\"{}{}\">{}</a>",
                  entry_name, is_dir,
                  entry_name));
        }
        content.push_str("</ul></body></html>");
        res.send(content.as_bytes()).unwrap();
    } else {
    }
}

fn simple_http_server(req: Request, mut res: Response) {
    match req.uri {
        AbsolutePath(ref path) => match &req.method {
            &Get => {
                show_file(path, res);
                return;
            },
            &Post => {
                return;
            }
            _ => {
                *res.status_mut() = hyper::NotFound;
                return;
            }
        },
        _ => {
            *res.status_mut() = hyper::NotFound;
            return;
        }
    }
}

fn main() {
    env_logger::init().unwrap();
    /* for https
    let ssl_context =
      Openssl::with_cert_and_key("/home/ytshen/proj/rust/rust_exp/cert.pem",
          "/home/ytshen/proj/rust/rust_exp/key.pem").unwrap();
    let server = Server::https("0.0.0.0:8443", ssl_context).unwrap();
    */
    let server = Server::http("0.0.0.0:8001").unwrap();
    let _guard = server.handle(simple_http_server);
    println!("Listening on http://0.0.0.0:8000");
}
