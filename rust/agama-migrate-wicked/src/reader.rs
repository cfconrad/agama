use crate::interface::Interface;
use quick_xml::de::from_str;
use regex::Regex;
use std::fs;
use std::io;
use std::path::PathBuf;

pub fn read_xml(path: PathBuf) -> Result<Interface, quick_xml::DeError> {
    let contents = fs::read_to_string(path).expect("Should have been able to read the file");
    // TODO better error handling when xml parsing failed
    let interface: Interface = from_str(replace_colons(contents).as_str())?;
    Ok(interface)
}

fn replace_colons(colon_string: String) -> String {
    let re = Regex::new(r"<([\/]?)(\w+):(\w+)\b").unwrap();
    let replaced = re
        .replace_all(colon_string.as_str(), "<$1$2-$3")
        .to_string();
    replaced
}

pub async fn read_dir(path: PathBuf) -> Result<Vec<Interface>, io::Error> {
    let interfaces = fs::read_dir(path)?
        .filter(|r| !r.as_ref().unwrap().path().is_dir())
        .map(|res| res.map(|e| read_xml(e.path()).unwrap()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    Ok(interfaces)
}
