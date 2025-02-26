use std::error::Error;

#[derive(Debug)]
pub enum NetError {
    InvalidNetworkAddress
}
impl std::fmt::Display for NetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for NetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

/// Parses a proxy authentication string in the format `username:password`.
///
/// This function splits the input string at the colon (`:`) to extract the username and password.
/// If the string does not contain exactly one colon, an error is returned.
///
/// # Arguments
///
/// * `s` - A string slice containing the proxy authentication details in `username:password` format.
///
/// # Returns
///
/// Returns a `Result` containing a tuple `(String, String)` with the extracted username and password,
/// or a `NetError::InvalidNetworkAddress` if the format is incorrect.
///
/// # Errors
///
/// * Returns `NetError::InvalidNetworkAddress` if the input string does not contain exactly one `:` separator.
///
pub fn parse_proxy_auth(s: &str) -> Result<(String, String), NetError> {
    let parts: Vec<_> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(NetError::InvalidNetworkAddress);
    }

    let user = parts[0].to_string();
    let passwd = parts[1].to_string();

    Ok((user, passwd))
}

  