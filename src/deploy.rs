use std::path::PathBuf;
use std::process::Command;

use Destination;
use EnokeysError;

pub fn deploy(destinations: &[Destination], file: &PathBuf) -> Result<(), EnokeysError> {
    for destination in destinations {
        Command::new("scp")
            .args(&[
                "-P",
                &format!("{}", destination.port),
                "-o",
                "StrictHostKeyChecking=no",
                &file.to_str().unwrap().to_string(),
                &format!(
                    "{}@{}:~/.ssh/authorized_keys",
                    &destination.userauth_agent, &destination.address
                ),
            ])
            .status()?;
    }
    Ok(())
}
