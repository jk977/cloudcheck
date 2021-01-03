use std::net::Ipv4Addr;
use std::str::FromStr;

/**
 * Get the `n`th field in `s`, using whitespace as a delimiter.
 */
fn get_field(s: &str, n: usize) -> Option<&str> {
    s.split_whitespace().nth(n)
}

/**
 * Check if `log` is an sshd log.
 */
fn log_is_sshd(log: &str) -> bool {
    const SERVICE_IDX: usize = 4;
    get_field(log, SERVICE_IDX)
        .map(|s| s.starts_with("sshd["))
        .unwrap_or(false)
}

/**
 * Check if `log` is an sshd login failure.
 */
fn log_is_sshd_failure(log: &str) -> bool {
    const MSG_START_IDX: usize = 5;
    log_is_sshd(log) && get_field(log, MSG_START_IDX) == Some("Invalid")
}

pub enum LogError {
    BadFormat,
    BadValue,
}

pub struct SshdEvent {
    pub log: String,
    pub user: String,
    pub addr: Ipv4Addr,
    pub port: u16,
}

impl FromStr for SshdEvent {
    type Err = LogError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use LogError::*;

        if !log_is_sshd_failure(s) {
            return Err(BadFormat);
        }

        Ok(SshdEvent {
            log: s.to_string(),
            user: get_field(s, 7)
                .ok_or(BadFormat)?
                .to_string(),
            addr: get_field(s, 9)
                .ok_or(BadFormat)?
                .parse()
                .map_err(|_| BadValue)?,
            port: get_field(s, 11)
                .ok_or(BadFormat)?
                .parse()
                .map_err(|_| BadValue)?,
        })
    }
}
