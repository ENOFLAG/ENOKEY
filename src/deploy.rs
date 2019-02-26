use std::process::Command;

use Destination;
use EnokeysError;


pub fn deploy(destinations: &[Destination]) -> Result<(), EnokeysError> {
    for destination in destinations {
        Command::new("scp")
            .args(&["-P", &format!("{}", destination.port),
                "-o", "StrictHostKeyChecking=no",
                &destination.authorized_keys_file_name.to_str().unwrap().to_string(), &format!("{}@{}:~/.ssh/authorized_keys", &destination.userauth_agent, &destination.address)])
            .status()?;
    }
    Ok(())
}
