extern crate exif;

use std::env;
use std::fmt::Write;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use exif::{Value, Tag, DateTime};

use std::f32;

fn main() {
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
    for f in reader.fields() {
        let thumb = if f.thumbnail { "1/" } else { "0/" };
        println!("  {}{}: {}", thumb, f.tag, f.value.display_as(f.tag));
        if let exif::Value::Ascii(ref s) = f.value {
            println!("      Ascii({:?})",
                     s.iter().map(escape).collect::<Vec<_>>());
        } else {
            println!("      {:?}", f.value);
        }
    }
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
    if let Some(field) = reader.get_field(Tag::GPSLatitude, false) {
        let mut buf = String::new();
        write!(buf, "{}", field.value.display_as(field.tag));
        println!("{}: {}", field.tag, field.value.display_as(field.tag));
        let v = buf.split(" ").collect::<Vec<&str>>();
        println!("{} {} {}", v[0], v[2], v[4]);
        let deg: f32 = (v[0].parse::<f32>().unwrap()) + (v[2].parse::<f32>().unwrap() / 60.0) + (v[4].parse::<f32>().unwrap() / 3600.0);
        println!("> {}", deg);
    }
    if let Some(field) = reader.get_field(Tag::GPSLongitude, false) {
        let mut buf = String::new();
        write!(buf, "{}", field.value.display_as(field.tag));
        println!("{}: {}", field.tag, field.value.display_as(field.tag));
        let v = buf.split(" ").collect::<Vec<&str>>();
        println!("{} {} {}", v[0], v[2], v[4]);
        let deg: f32 = (v[0].parse::<f32>().unwrap()) + (v[2].parse::<f32>().unwrap() / 60.0) + (v[4].parse::<f32>().unwrap() / 3600.0);
        println!("> {}", deg);
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
