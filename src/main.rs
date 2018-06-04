extern crate exif;
extern crate iron;
extern crate iron_cors;
extern crate glob;
#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

use std::env;
use std::fmt::Write;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use exif::{Value, Tag, DateTime};
use std::io::{self, Read};

use iron::prelude::*;
use iron::headers::ContentType;
use iron::Handler;
use iron::status;
use iron_cors::CorsMiddleware;

use std::collections::HashMap;

use glob::glob;

#[derive(Serialize,Deserialize,Debug)]
struct Photo {
    path: String,
    timestamp: String,
    width: u32,
    height: u32,
}

enum Error {
    ParsePhotoError(String),
    OtherError(String)
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::OtherError("io error".into())
    }
}

impl From<exif::Error> for Error {
    fn from(error: exif::Error) -> Self {
        Error::OtherError("exif error".into())
    }
}

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
        let mut photos: Vec<Photo> = vec![];
        for entry in glob("./photos/*").expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => match parse_file(path) {
                    Ok(photo) => photos.push(photo),
                    _ => println!("error parsing photo"),
                },
                Err(e) => println!("{:?}", e),
            }
        }
        let s = serde_json::to_string(&photos).unwrap();
        Ok(Response::with((ContentType::json().0, status::Ok, s)))
    });

    let mut chain = Chain::new(router);
    let cors_middleware = CorsMiddleware::with_allow_any();
    chain.link_around(cors_middleware);

    Iron::new(chain).http("localhost:3004").unwrap();
}

fn parse_file(path: PathBuf) -> Result<Photo, Error> {
    let file = try!(File::open(&path));
    let reader = try!(exif::Reader::new(&mut BufReader::new(&file)));
    let mut datetime = String::new();
    let mut width = 0;
    let mut height = 0;

    println!("{}", path.display());

    if let Some(field) = reader.get_field(Tag::DateTime, false) {
        match field.value {
            Value::Ascii(ref vec) if !vec.is_empty() => {
                if let Ok(d) = DateTime::from_ascii(vec[0]) {
                    println!("Year of DateTime is {}.", d.year);
                    datetime = d.to_string();
                }
            },
            _ => {},
        }
    }
    if let Some(field) = reader.get_field(Tag::PixelXDimension, false) {
        if let Some(w) = field.value.get_uint(0) {
            println!("Valid width of the image is {}.", w);
            width = w;
        }
    }
    if let Some(field) = reader.get_field(Tag::PixelYDimension, false) {
        if let Some(h) = field.value.get_uint(0) {
            println!("Valid height of the image is {}.", h);
            height = h;
        }
    }

    Ok(Photo {
        path: path.into_os_string().into_string().unwrap(),
        timestamp: datetime,
        width: width,
        height: height,
        })
}