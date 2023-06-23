/// Bytes to indicate Flash Player version/Flash Media Server version.
///
/// This is used for indicating whether doing handshake with HMAC-SHA256 digest/signature.
/// If you do handshake with HMAC-SHA256 as a client, set major version and above 9.
/// If you do it as a server, set major version and above 3.
/// If otherwise, set major version below 9/3, or you can set 0.
///
/// Because of handshake specification, note any value above `0xff` cannot set as a version. Such as a last byte of Flash Player version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version(u8, u8, u8, u8);

impl Version {
    /// Bytes meant not to use HMAC-SHA256.
    pub const UNSIGNED: Self = Self(0, 0, 0, 0);
    /// The latest version of Flash Player.
    pub const LATEST_CLIENT: Self = Self(32, 0, 0, 0);
    /// The latest version of Flash Media Server.
    pub const LATEST_SERVER: Self = Self(5, 0, 17, 0);

    /// Gets a number of major version either Flash Player or Flash Media Server.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sheave_core::handshake::Version;
    ///
    /// assert_eq!(0, Version::UNSIGNED.get_major_version());
    /// assert_eq!(32, Version::LATEST_CLIENT.get_major_version());
    /// assert_eq!(5, Version::LATEST_SERVER.get_major_version())
    /// ```
    ///
    /// The well-known RTMP applications check just this version to decide whether they do handshake with HMAC-SHA256.
    pub fn get_major_version(&self) -> u8 {
        self.0
    }
}

impl From<[u8; 4]> for Version {
    fn from(version_bytes: [u8; 4]) -> Self {
        Self(version_bytes[0], version_bytes[1], version_bytes[2], version_bytes[3])
    }
}

impl From<Version> for [u8; 4] {
    fn from(version: Version) -> Self {
        let mut version_bytes: [u8; 4] = [0; 4];
        version_bytes[0] = version.0;
        version_bytes[1] = version.1;
        version_bytes[2] = version.2;
        version_bytes[3] = version.3;
        version_bytes
    }
}
