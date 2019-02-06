use std::net::TcpStream;
use ssh2::Session;
use std::path::Path;
use std::io::Write;
use std::io::Read;
use std::fs::File;

use Destination;
use EnokeysError;


pub fn deploy(destinations: &Vec<Destination>) -> Result<(), EnokeysError> {
    for destination in destinations {
        let file_name = format!("keyfiles/{}.authorized_keys",destination.destination_name);
        let mut content = vec!();
        File::open(file_name)?.read_to_end(&mut content)?;
        let tcp = TcpStream::connect(&destination.address)?;
        let mut sess = Session::new().unwrap();
        sess.handshake(&tcp)?;
        sess.userauth_agent(&destination.userauth_agent)?;
        let mut remote_file = sess.scp_send(Path::new("~/.ssh/authorized_keys"), 0o644, content.len() as u64, None)?;
        remote_file.write_all(&content)?;
    }
    Ok(())
}
