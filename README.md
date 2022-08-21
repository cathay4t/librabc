# Example Linux System Library in Rust -- librabc

This is a example project to demonstrate my practise on linux
system library in Rust containing:
 * A echo server `rabcd` listening on UNIX socket `/tmp/librabc`.
 * Rust crate connect above socket and send `ping` every 10 seconds.
 * C/Python binding
 * Command line tool for the client `rabcc`.
