use std::process::Command;

use Destination;
use EnokeysError;


pub fn deploy(destinations: &Vec<Destination>) -> Result<(), EnokeysError> {
    for destination in destinations {
        Command::new("scp")
            .args(&["-i", "./data/id_ed25519",
                "-P", &format!("{}", destination.port),
                "-o", "StrictHostKeyChecking=no",
                &destination.authorized_keys_file_name.to_str().unwrap().to_string(), &format!("{}@{}:/home/{}/.ssh/authorized_keys", &destination.userauth_agent, &destination.address, &destination.userauth_agent)])
            .status()?;
    }
    Ok(())
}
