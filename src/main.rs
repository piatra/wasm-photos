extern crate exif;
extern crate glob;
#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Write;
use std::path::{PathBuf};
use exif::{Value, Tag, DateTime};
use std::io::{self};

use std::collections::HashMap;

use glob::glob;

#[derive(Serialize,Deserialize,Debug)]
struct Photo {
    path: String,
    timestamp: String,
    width: u32,
    height: u32,
    country: Country,
}

#[derive(Deserialize,Serialize,Debug)]
struct Country {
    name: String,
    location: Location,
}

#[derive(Serialize,Deserialize,Debug)]
struct Location {
    lat: f32,
    lng: f32,
}

enum Error {
    ParsePhotoError(String),
    OtherError(String)
}

impl From<io::Error> for Error {
    fn from(_error: io::Error) -> Self {
        Error::OtherError("io error".into())
    }
}

impl From<exif::Error> for Error {
    fn from(_error: exif::Error) -> Self {
        Error::OtherError("exif error".into())
    }
}

fn distance(a: &Location, b: &Location) -> f32 {
    (a.lat - b.lat) * (a.lat - b.lat) + (a.lng - b.lng) * (a.lng - b.lng)
}

fn get_photo_location(photo_location: &Location) -> String {
    let mut f = File::open("./countries_and_locations.json").expect("file not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect("Reading the countries list failed");
    let countries: Vec<Country> = serde_json::from_str(&contents).expect("Deserializing the countries list failed");
    let mut closest = distance(photo_location, &countries[0].location);
    let mut location = String::new();
    for country in countries {
        let compare_dist = distance(photo_location, &country.location);
        if  compare_dist <= closest {
            closest = compare_dist;
            location = country.name;
            println!("{} {}", location, closest);
        }
    }

    location
}

fn main() {
    let mut photos: Vec<Photo> = vec![];
    for entry in glob("./photos/*").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => match parse_file(path) {
                Ok(photo) => photos.push(photo),
                Err(e) => match e {
                    Error::ParsePhotoError(msg) => println!("Photo parsing error {}", msg),
                    Error::OtherError(msg) => println!("Other error {}", msg)
                }
            },
            Err(e) => println!("{:?}", e),
        }
    }
    let s = serde_json::to_string(&photos).unwrap();
    let mut file = File::create("output.json").unwrap();
    file.write_all(s.as_ref());
}

fn parse_file(path: PathBuf) -> Result<Photo, Error> {
    let file = try!(File::open(&path));
    let reader = try!(exif::Reader::new(&mut BufReader::new(&file)));
    let mut datetime = String::new();
    let mut width = 0;
    let mut height = 0;
    let mut lat = 0.0;
    let mut lng = 0.0;

    if let Some(field) = reader.get_field(Tag::DateTime, false) {
        match field.value {
            Value::Ascii(ref vec) if !vec.is_empty() => {
                if let Ok(d) = DateTime::from_ascii(vec[0]) {
                    datetime = d.to_string();
                } else {
                    return Err(Error::ParsePhotoError("Datetime".into()))
                }
            },
            _ => return Err(Error::ParsePhotoError("Datetime".into())),
        }
    }
    if let Some(field) = reader.get_field(Tag::PixelXDimension, false) {
        if let Some(w) = field.value.get_uint(0) {
            width = w;
        } else {
            return Err(Error::ParsePhotoError("Dimensions".into()))
        }
    }
    if let Some(field) = reader.get_field(Tag::PixelYDimension, false) {
        if let Some(h) = field.value.get_uint(0) {
            height = h;
        } else {
            return Err(Error::ParsePhotoError("Dimensions".into()))
        }
    }
     if let Some(field) = reader.get_field(Tag::GPSLatitude, false) {
        let buf = format!("{}", field.value.display_as(field.tag));
        let v = buf.split(" ").collect::<Vec<&str>>();
        lat = (v[0].parse::<f32>().unwrap()) + (v[2].parse::<f32>().unwrap() / 60.0) + (v[4].parse::<f32>().unwrap() / 3600.0);
    }
    if let Some(field) = reader.get_field(Tag::GPSLongitude, false) {
        let buf = format!("{}", field.value.display_as(field.tag));
        let v = buf.split(" ").collect::<Vec<&str>>();
        lng = (v[0].parse::<f32>().unwrap()) + (v[2].parse::<f32>().unwrap() / 60.0) + (v[4].parse::<f32>().unwrap() / 3600.0);
    }

    let location = Location {
            lat: lat,
            lng: lng,
        };

    Ok(Photo {
        path: path.into_os_string().into_string().unwrap(),
        timestamp: datetime,
        width: width,
        height: height,
        country: Country {
            name: get_photo_location(&location),
            location,
            }
        })
}
