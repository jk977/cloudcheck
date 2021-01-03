use std::net::Ipv4Addr;
use std::str::FromStr;

const SERVICE_FIELD: usize = 4;
const MSG_FIELD: usize = 5;
const USER_FIELD: usize = 7;
const ADDRESS_FIELD: usize = 9;
const PORT_FIELD: usize = 11;

pub enum LogError {
    BadFormat,
    BadValue,
}

/**
 * Get the `n`th field in `s`, using whitespace as a delimiter.
 */
fn get_field(s: &str, n: usize) -> Result<&str, LogError> {
    s.split_whitespace().nth(n).ok_or(LogError::BadFormat)
}

/**
 * Parse the `n`th field in `s`, using whitespace as a delimiter.
 */
fn parse_field<T: FromStr>(s: &str, n: usize) -> Result<T, LogError> {
    get_field(s, n)?.parse().map_err(|_| LogError::BadValue)
}

/**
 * Apply `predicate` to the `n`th whitespace-separated field in `s`.
 */
fn check_field<F>(s: &str, n: usize, predicate: F) -> bool
    where F: Fn(&str) -> bool
{
    get_field(s, n).map(predicate).unwrap_or(false)
}

/**
 * Check if `log` is an sshd authentication failure.
 */
fn log_is_sshd_failure(log: &str) -> bool {
    check_field(log, SERVICE_FIELD, |s| s.starts_with("sshd["))
        && check_field(log, MSG_FIELD, |s| s == "Invalid")
}

#[derive(Debug)]
pub struct SshdEvent {
    pub log: String,
    pub user: String,
    pub addr: Ipv4Addr,
    pub port: u16,
}

impl FromStr for SshdEvent {
    type Err = LogError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !log_is_sshd_failure(s) {
            return Err(LogError::BadFormat);
        }

        Ok(SshdEvent {
            log: s.to_string(),
            user: get_field(s, USER_FIELD)?.to_string(),
            addr: parse_field(s, ADDRESS_FIELD)?,
            port: parse_field(s, PORT_FIELD)?,
        })
    }
}
