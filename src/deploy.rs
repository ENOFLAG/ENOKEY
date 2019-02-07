use std::process::Command;

use Destination;
use EnokeysError;


pub fn deploy(destinations: &Vec<Destination>) -> Result<(), EnokeysError> {
    for destination in destinations {
        let file_name = format!("keyfiles/{}.authorized_keys",destination.authorized_keys_file_name);
        Command::new("scp")
            .args(&["-i", "./data/id_ed25519",
                "-P", &format!("{}", destination.port),
                "-o", "StrictHostKeyChecking=no",
                &file_name, &format!("{}@{}:/home/{}/.ssh/authorized_keys", &destination.userauth_agent, &destination.address, &destination.userauth_agent)])
            .status()?;
    }
    Ok(())
}
