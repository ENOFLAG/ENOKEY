use std::io::Write;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::cmp::min;

use openssh_keys::PublicKey;

use error::EnokeysError;
use scraper;
use USERNAME_REGEX;
use Destination;


pub fn handle_raw_submission(name: &str, pub_key: &str, destinations: &Vec<Destination>) -> Result<(), EnokeysError> {
    for destination in destinations {
        let name = USERNAME_REGEX.replace_all(name, " ");
        let raw_storage_file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(format!("./data/{}.storage.raw", destination.destination_name))?;
        writeln!(&raw_storage_file, "{} {}@raw", &pub_key, &name)?;
    }
    Ok(())
}

pub fn handle_submission(provider: &str, user_name: &str, name: &str, destinations: &Vec<Destination>) -> Result<(), EnokeysError> {
    if provider.is_empty() || user_name.is_empty() {
        return Err(EnokeysError::InvalidData("username or service empty".to_string()));
    }
    let user_name = USERNAME_REGEX.replace_all(user_name, "");
    let name = USERNAME_REGEX.replace_all(name, " ");
    for destination in destinations {
        let mut storage_file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(format!("./data/{}.storage", &destination.destination_name))?;
        let line = format!("# {} \n{}:{}\n", &name, provider, &user_name);
        println!("Adding entry:\n{}", &line);
        write!(storage_file, "{}", &line)?;
    }
    Ok(())
}

pub fn generate_authorized_key_files(destinations: &Vec<Destination>) -> Result<(), EnokeysError> {
    for destination in destinations {
        let mut authorized_keys_file = File::create(format!("./keyfiles/{}.authorized_keys", &destination.destination_name))?;
        let mut authorized_keys_file_content = String::new();
        let mut storage_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(format!("./data/{}.storage", destination.destination_name))?;
        let mut storage_file_content = String::new();

        // append raw keys
        let mut raw_keys = String::new();
        if let Ok(mut raw_keys_file) = File::open(format!("./data/{}.storage.raw", destination.destination_name)) {
            raw_keys_file.read_to_string(&mut raw_keys)?;
            authorized_keys_file_content.push_str(&raw_keys);
            write!(authorized_keys_file, "{}", &raw_keys)?
        }

        // append keys from providers
        storage_file.read_to_string(&mut storage_file_content)?;
        for line in storage_file_content.split('\n').filter(|&i|!i.is_empty()).filter(|&s|&s[0..1]!="#") {
            let entry = line.split(':').collect::<Vec<&str>>();
            let user_keys = scraper::fetch(entry[1], entry[0])?;
            for key in user_keys {
                println!("parsing key: {}", &key);
                match PublicKey::parse(&key) {
                    Ok(key) => {
                        match &key.comment {
                            Some(ref comment) => {
                                let comment = USERNAME_REGEX.replace_all(&comment, " ");
                                let line = format!("{} {} {}\n", key.keytype(), base64::encode(&key.data()), &comment[0..min(comment.len(), 100)]);
                                authorized_keys_file_content.push_str(&line);
                                write!(authorized_keys_file, "{}", &line)?
                            },
                            None => writeln!(authorized_keys_file, "{} {}", key.keytype(), base64::encode(&key.data()))?
                        }
                    },
                    Err(e) => println!("Failed to parse PublicKey: {:?}", e)
                }
            }
        }
    }
    Ok(())
}
