extern crate exif;

use std::env;
use std::fmt::Write;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

fn main() {
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
