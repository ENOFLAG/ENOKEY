use std::net::TcpStream;
use ssh2::Session;
use std::path::Path;
use std::io::Write;
use Config;


pub fn deploy(_cfg: &Config, content: &str) {
    let tcp = TcpStream::connect("localhost:22").unwrap();
    let mut sess = Session::new().unwrap();
    sess.handshake(&tcp).unwrap();
    sess.userauth_agent("fh").unwrap();

    let mut remote_file = sess.scp_send(Path::new("/tmp/authorized_keys"),
                                        0o644, 10, None).unwrap();
    remote_file.write(content.as_bytes()).unwrap();
}