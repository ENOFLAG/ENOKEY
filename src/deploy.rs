use std::net::TcpStream;
use ssh2::Session;
use std::path::Path;
use std::io::Write;

use Destination;
use EnokeysError;


pub fn deploy(content: &str, destination: &Destination) -> Result<(), EnokeysError> {
    let tcp = TcpStream::connect(&destination.address)?;
    let mut sess = Session::new().unwrap();
    sess.handshake(&tcp)?;
    sess.userauth_agent(&destination.userauth_agent)?;
    let mut remote_file = sess.scp_send(Path::new("/root/.ssh/authorized_keys"), 0o644, content.len() as u64, None)?;
    remote_file.write_all(content.as_bytes())?;
    Ok(())
}
