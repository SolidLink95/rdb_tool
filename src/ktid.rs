use std::fmt;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct KTID(pub u32);

impl KTID {
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    /// The argument is treated as a path, meaning you need to provide the filename and extension like a regular path.
    #[allow(dead_code)]
    pub fn new<P: AsRef<Path>>(name: P) -> Self {
        KTID::from(name.as_ref())
    }
}

impl From<u32> for KTID {
    fn from(hash: u32) -> Self {
        KTID(hash)
    }
}

impl From<&str> for KTID {
    fn from(string: &str) -> Self {
        ktid(string)
    }
}

impl FromStr for KTID {
    type Err = ();
    fn from_str(s: &str) -> Result<KTID, ()> {
        Ok(KTID::from(s))
    }
}

impl From<&Path> for KTID {
    fn from(path: &Path) -> Self {
        let mut buffer = Vec::<u8>::new();
        buffer.extend_from_slice("R_".as_bytes());
        // Huehuehue
        buffer.extend_from_slice(
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|s| s.as_bytes())
                .expect(&format!("Invalid extension: \"{:?}\"", path.extension()))
        );
        buffer.extend_from_slice("［".as_bytes());
        // buffer.extend_from_slice(path.file_stem().unwrap().to_str().unwrap().as_bytes());
        buffer.extend_from_slice(path.file_stem().and_then(|x| x.to_str()).and_then(|s| Some(s.as_bytes())).expect("Invalid file_stem"));
        buffer.extend_from_slice("］".as_bytes());

        ktid(&String::from_utf8(buffer).expect(&format!("Invalid path: {:?}", path)))
    }
}

impl fmt::Display for KTID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:08x}", self.as_u32())
    }
}

pub fn ktid(string: &str) -> KTID {
    if string.starts_with("0x") {
        KTID(u32::from_str_radix(string.trim_start_matches("0x"), 16).expect("Invalid KTID"))

    } else {
        KTID(ktid_hash(string, 31))
    }
}

pub fn ktid_hash<T: AsRef<[u8]>>(text: T, mut key: i32) -> u32 {
    let bytes = text.as_ref();

    let mut iv = bytes[0] as i32 * 31;

    for cur_char in &bytes[1..] {
        iv = iv.wrapping_add(31i32.wrapping_mul(key.wrapping_mul((*cur_char as i8) as i32)));
        key = key.wrapping_mul(31);
    }

    iv as u32
}
