use serde_json::Value;
use std::collections::HashMap;
use std::io::Read;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt()]
    subcommand: Subcommand,
    #[structopt(short, long)]
    key: Option<String>,
    #[structopt(short, long)]
    value: Option<String>,
}

#[derive(Debug)]
enum Subcommand {
    Set,
    Get,
    Delete,
}

impl FromStr for Subcommand {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "set" => Ok(Subcommand::Set),
            "get" => Ok(Subcommand::Get),
            "delete" => Ok(Subcommand::Delete),
            _ => Err("Subcommand must be either  'get' or 'set' or 'delete'".to_string()),
        }
    }
}

// data [subcommand=set|get|get] [key] {value}
fn main() {
    let opt = Opt::from_args();
    println!("{:#?}", opt);

    match opt.subcommand {
        Subcommand::Get => get(opt.key).unwrap(),
        Subcommand::Set => set(opt.key, opt.value).unwrap(),
        Subcommand::Delete => delete(opt.key, opt.value).unwrap(),
    }
}

fn get(key: Option<String>) -> std::io::Result<()> {
    println!("getting keys");
    let map = load_keys()?;

    if key.is_some() {
        println!("{:#?}", map.get(&key.unwrap()).unwrap());
    } else {
        println!("{:#?}", map);
    }

    Ok(())
}

fn set(key: Option<String>, value: Option<String>) -> std::io::Result<()> {
    if key.is_some() && value.is_some() {
        let mut map = load_keys()?;
        map.insert(key.unwrap(), value.unwrap());
        write_keys(&map)?;
    } else {
        println!("missing key or value")
    }

    Ok(())
}

fn delete(key: Option<String>, value: Option<String>) -> std::io::Result<()> {
    let key_exists = key.is_some();
    let value_exists = value.is_some();
    let str_key = &key.unwrap_or(String::from(""));
    let str_value = &value.unwrap_or(String::from(""));

    let mut map = if key_exists || value_exists {
        load_keys()?
    } else {
        HashMap::new()
    };

    if key_exists {
        if map.contains_key(str_key) {
            map.remove(str_key);
            write_keys(&map)?;
        }
    }

    if value_exists {
        let to_remove = map
            .keys()
            .find(|key| map.get(*key).unwrap() == str_value)
            .cloned();

        if to_remove.is_some() {
            map.remove(&to_remove.unwrap());
            write_keys(&map)?;
        }
    }

    Ok(())
}

fn write_keys(map: &HashMap<String, String>) -> std::io::Result<()> {
    let jstr = serde_json::to_string(&map)?;
    std::fs::write("data.db", jstr.as_bytes())?;

    Ok(())
}

fn load_keys() -> std::io::Result<HashMap<String, String>> {
    let mut file = match std::fs::File::open("data.db") {
        Ok(file) => file,
        Err(ref err) if err.kind() == std::io::ErrorKind::NotFound => {
            std::fs::File::create("data.db")?
        }
        Err(e) => return Err(e),
    };

    let mut content = String::new();
    file.read_to_string(&mut content)?;
    if content.is_empty() {
        content.push_str("{}");
    }
    let json: Value = serde_json::from_str(&content)?;
    match json {
        Value::Object(map) => {
            let mut db = HashMap::new();
            for (k, value) in map {
                match value {
                    Value::String(string) => db.insert(k, string),
                    _ => panic!("Bad Map: CORRUPT DATABASE!!!"),
                };
            }
            Ok(db)
        }
        _ => panic!("Not a Map: CORRUPT DATABASE!!!"),
    }
}
