extern crate exif;
extern crate iron;
extern crate glob;

use std::env;
use std::fmt::Write;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use exif::{Value, Tag, DateTime};

use iron::prelude::*;
use iron::headers::ContentType;
use iron::Handler;
use iron::status;

use std::collections::HashMap;

use glob::glob;

fn hello_world(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((iron::status::Ok, "Hello World")))
}

struct Router {
    // Routes here are simply matched with the url path.
    routes: HashMap<String, Box<Handler>>
}

impl Router {
    fn new() -> Self {
        Router { routes: HashMap::new() }
    }

    fn add_route<H>(&mut self, path: String, handler: H) where H: Handler {
        self.routes.insert(path, Box::new(handler));
    }
}

impl Handler for Router {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        match self.routes.get(&req.url.path().join("/")) {
            Some(handler) => handler.handle(req),
            None => Ok(Response::with(status::NotFound))
        }
    }
}

fn main() {
    let mut router = Router::new();
    router.add_route("hello".to_string(), |_: &mut Request| {
        Ok(Response::with((status::Ok, "Hello Photos")))
    });
    router.add_route("photos".to_string(), |_: &mut Request| {
        let mut photos: Vec<String> = vec![];
        for entry in glob("./photos/*").expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => { println!("{:?}", path.display()); photos.push(path.into_os_string().into_string().unwrap()) },
                Err(e) => println!("{:?}", e),
            }
        }
        let mut contents: String = String::new();
        println!("{:?}", photos);
        write!(contents, "{{ \"photos\": [\"{}\"] }}", photos.join(","));
        Ok(Response::with((ContentType::json().0, status::Ok, contents)))
    });


    Iron::new(router).http("localhost:3004").unwrap();
}

fn read_photo() {
    let path = Path::new("./photos");
    for entry in path.read_dir().expect("read_dir call failed") {
        if let Ok(entry) = entry {
            if let Err(e) = dump_file(&entry.path()) {
                println!("{}: {}", path.display(), e);
            }
        }
    }
    for path in env::args_os().skip(1).map(PathBuf::from) {
        if let Err(e) = dump_file(&path) {
            println!("{}: {}", path.display(), e);
        }
    }
}

fn dump_file(path: &Path) -> Result<(), exif::Error> {
    let file = try!(File::open(path));
    let reader = try!(exif::Reader::new(&mut BufReader::new(&file)));

    println!("{}", path.display());
    // for f in reader.fields() {
    //     let thumb = if f.thumbnail { "1/" } else { "0/" };
    //     println!("  {}{}: {}", thumb, f.tag, f.value.display_as(f.tag));
    //     if let exif::Value::Ascii(ref s) = f.value {
    //         println!("      Ascii({:?})",
    //                  s.iter().map(escape).collect::<Vec<_>>());
    //     } else {
    //         println!("      {:?}", f.value);
    //     }
    // }
    if let Some(field) = reader.get_field(Tag::DateTime, false) {
        match field.value {
            Value::Ascii(ref vec) if !vec.is_empty() => {
                if let Ok(datetime) = DateTime::from_ascii(vec[0]) {
                    println!("Year of DateTime is {}.", datetime.year);
                }
            },
            _ => {},
        }
    }
    if let Some(field) = reader.get_field(Tag::PixelXDimension, false) {
        if let Some(width) = field.value.get_uint(0) {
            println!("Valid width of the image is {}.", width);
        }
    }
    if let Some(field) = reader.get_field(Tag::PixelYDimension, false) {
        if let Some(height) = field.value.get_uint(0) {
            println!("Valid height of the image is {}.", height);
        }
    }
    Ok(())
}

fn escape(bytes: &&[u8]) -> String {
    let mut buf = String::new();
    for &c in *bytes {
        match c {
            b'\\' | b'"' => write!(buf, "\\{}", c as char).unwrap(),
            0x20...0x7e => buf.write_char(c as char).unwrap(),
            _ => write!(buf, "\\x{:02x}", c).unwrap(),
        }
    }
    buf
}
