extern crate exif;
extern crate glob;
extern crate image;

use image::{FilterType, PNG, GenericImageView};

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
    date: PhotoDate,
    width: u32,
    height: u32,
    country: Country,
}

#[derive(Serialize,Deserialize,Debug)]
struct PhotoDate {
    timestamp: String,
    year: u16,
    month: u8,
    day: u8,
}

impl PhotoDate {
    fn new(year: u16, month: u8, day: u8, time_str: &str) -> PhotoDate {
        PhotoDate {
            timestamp: time_str.to_owned(),
            year,
            month,
            day
        }
    }
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

fn resize(path: PathBuf) -> () {
    match image::open(&path) {
        Ok(img) => {
            let (width, height) = img.dimensions();
            let ratio: f32 = if width > height {
                300.0 / width as f32
            } else {
                300.0 / height as f32
            };
            let scaled = img.resize((width as f32 * ratio) as u32, (height as f32 * ratio) as u32, FilterType::Gaussian);
            let name = path.into_os_string().into_string().unwrap();
            let mut output = File::create(&format!("test-{}.png", name)).unwrap();
            scaled.write_to(&mut output, PNG).unwrap();
        }
        _ => println!("Could not find {:?}", path)
    }
}

fn get_photo_location(photo_location: &Location) -> String {
    let mut f = File::open("./countries_and_locations.json").unwrap();
    let mut contents = String::new();
    f.read_to_string(&mut contents).unwrap();
    let countries: Vec<Country> = serde_json::from_str(&contents).unwrap();
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
    let mut photos: HashMap<String, Vec<Photo>> = HashMap::new();
    for entry in glob("./photos/*").expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => match parse_file(path.clone()) {
                Ok(photo) => {
                    resize(path);
                    let key = photo.country.name.to_owned() + &photo.date.year.to_string();
                    photos.entry(key)
                        .or_insert_with(Vec::new)
                        .push(photo)
                },
                _ => println!("error parsing photo"),
            },
            Err(e) => println!("{:?}", e),
        }
    }
    let s = serde_json::to_string(&photos).unwrap();
    let mut file = File::create("output.json").unwrap();
    if let Err(e) = file.write_all(s.as_ref()) {
        eprintln!("Error trying to write to output {:?}", e);
    }
}

fn to_f32(s: &str) -> f32 {
    s.parse().unwrap()
}

fn parse_file(path: PathBuf) -> Result<Photo, Error> {
    let file = File::open(&path)?;
    let reader = exif::Reader::new(&mut BufReader::new(&file))?;
    let datetime;
    let mut width = 0;
    let mut height = 0;
    let mut lat = 0.0;
    let mut lng = 0.0;

    if let Some(field) = reader.get_field(Tag::DateTime, false) {
        match field.value {
            Value::Ascii(ref vec) if !vec.is_empty() => {
                if let Ok(d) = DateTime::from_ascii(vec[0]) {
                    datetime = PhotoDate::new(d.year, d.month, d.day, &d.to_string());
                } else {
                    return Err(Error::ParsePhotoError("Datetime".into()));
                }
            },
            _ => return Err(Error::ParsePhotoError("Datetime".into())),
        }
    } else {
        return Err(Error::ParsePhotoError("Datetime".into()));
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
        lat = to_f32(v[0]) + (to_f32(v[2]) / 60.0) + (to_f32(v[4]) / 3600.0);
    }
    if let Some(field) = reader.get_field(Tag::GPSLongitude, false) {
        let buf = format!("{}", field.value.display_as(field.tag));
        let v = buf.split(" ").collect::<Vec<&str>>();
        lng = to_f32(v[0]) + (to_f32(v[2]) / 60.0) + (to_f32(v[4]) / 3600.0);
    }

    let location = Location {
            lat: lat,
            lng: lng,
        };

    Ok(Photo {
        path: path.into_os_string().into_string().unwrap(),
        date: datetime,
        width: width,
        height: height,
        country: Country {
            name: get_photo_location(&location),
            location,
            }
        })
}
