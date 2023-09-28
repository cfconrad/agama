use crate::interface::Interface;
use agama_dbus_server::network::model::ParentKind;
use quick_xml::de::from_str;
use regex::Regex;
use std::collections::HashMap;
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

pub fn post_process_interface(interfaces: &mut Vec<Interface>){
    let mut helper = HashMap::new();
    for (idx,i) in interfaces.iter().enumerate() {
        if let Some(parent) = &i.link.parent {
            for j in interfaces.iter() {
                if j.name == *parent {
                    if let Some(_) = &j.bond {
                        helper.insert(idx, Some(ParentKind::Bond));
                    }
                }
            }
        }
    }
    for (_, (k, v)) in helper.iter().enumerate() {
        if let Some(ifc) = interfaces.get_mut(*k) {
            (*ifc).link.kind = v.clone();
        }
    }
}

pub async fn read_dir(path: PathBuf) -> Result<Vec<Interface>, io::Error> {
    let mut interfaces = fs::read_dir(path)?
        .filter(|r| !r.as_ref().unwrap().path().is_dir())
        .map(|res| res.map(|e| read_xml(e.path()).unwrap()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    post_process_interface(&mut interfaces);
    Ok(interfaces)
}
