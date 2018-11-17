use std::io::Write;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::cmp::min;

use openssh_keys::PublicKey;

use error::EnokeysError;
use deploy;
use scraper;
use USERNAME_REGEX;
use Destination;


pub fn handle_submission(provider: &str, user_name: &str, name: &str, destination: &Destination) -> Result<(), EnokeysError> {
    if provider.is_empty() || user_name.is_empty() {
        return Err(EnokeysError::InvalidData("username or service empty".to_string()));
    }
    let user_name = USERNAME_REGEX.replace_all(user_name, "");
    let name = USERNAME_REGEX.replace_all(name, " ");
    save_storage(provider, &user_name, &name, destination)?;
    let key_file_content = save_authorized_key_file(&destination)?;
    deploy::deploy(&key_file_content, &destination)?;
    Ok(())
}

fn save_storage(provider: &str, user_name: &str, name: &str, destination: &Destination) -> Result<(), EnokeysError> {
    let mut storage_file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&destination.storage_file_name)?;
    let line = format!("# {} \n{}:{}\n", &name, provider, &user_name);
    println!("Adding entry:\n{}", &line);
    write!(storage_file, "{}", &line)?;
    Ok(())
}

fn save_authorized_key_file(destination: &Destination) -> Result<(String), EnokeysError> {
    let mut authorized_keys_file = File::create(&destination.authorized_keys_file_name)?;
    let mut authorized_keys_file_content = String::new();
    let mut storage_file = File::open(&destination.storage_file_name)?;
    let mut storage_file_content = String::new();
    storage_file.read_to_string(&mut storage_file_content)?;
    for line in storage_file_content.split('\n').filter(|&i|!i.is_empty()).filter(|&s|&s[0..1]!="#") {
        let entry = line.split(':').collect::<Vec<&str>>();
        let user_keys = scraper::fetch(entry[1], entry[0])?;
        for key in user_keys {
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
                Err(e) => println!("{:?}", e)
            }
        }
    }
    Ok(authorized_keys_file_content)
}
