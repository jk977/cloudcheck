use std::{
    fs::{self, File},
    io::{self, BufRead, BufReader},
    mem::MaybeUninit,
    net::Ipv4Addr,
    path::{Path, PathBuf},
    str,
};

use ipnet::Ipv4Net;

/**
 * Structure representing a JSON with host data.
 */
#[derive(Debug)]
pub struct HostJson {
    hostname: String,
    path: PathBuf,
    array_pointer: String,
    field: String,
}

impl HostJson {
    /**
     * Create a new structure containing information about a JSON containing host data.
     *
     * Params:
     *    - `hostname`: The name of the network host (e.g., "Google Cloud").
     *    - `path`: The path to the JSON file.
     *    - `array_pointer`: A JSON pointer to an array of objects within the JSON.
     *    - `field`: The name of the field containing the IPv4 range in each array object.
     */
    pub fn new(hostname: &str, path: &str, array_pointer: &str, field: &str) -> Self {
        Self {
            hostname: hostname.to_owned(),
            path: PathBuf::from(path),
            array_pointer: array_pointer.to_owned(),
            field: field.to_owned(),
        }
    }
}

/**
 * A network owned by a given host.
 */
struct HostNetwork {
    name: String,
    subnets: Vec<Ipv4Net>,
}

/**
 * A database associating host names with a network of IP addresses.
 */
pub struct HostDatabase {
    hosts: Vec<HostNetwork>,
}

impl HostDatabase {
    /**
     * Extract host data from the given JSONs.
     *
     * Returns `Ok(db)` if reading and parsing the JSONs succeeded, or an `io::Error` otherwise.
     */
    pub fn from_jsons(host_jsons: &[HostJson]) -> io::Result<Self> {
        let mut result = Self {
            hosts: Vec::with_capacity(host_jsons.len()),
        };

        for json in host_jsons {
            let current = HostNetwork {
                name: json.hostname.clone(),
                subnets: extract_from_json(json)?,
            };

            result.hosts.push(current);
        }

        Ok(result)
    }

    /**
     * Read hosts from the JSONs given in `csv`, a CSV with columns "HOSTNAME,PATH,POINTER,FIELD"
     * where HOSTNAME is the name of the host (e.g., "Google Cloud"), PATH is the path to the JSON
     * file containing IP ranges, POINTER is a JSON pointer to an array of objects in the CSV file,
     * and FIELD is the object field that contains the IP address.
     *
     * Returns: `Ok(db)` if reading the CSV and JSONs was successful, or an `io::Error` otherwise.
     */
    pub fn from_hosts_csv(csv: &Path) -> io::Result<Self> {
        let reader = BufReader::new(File::open(csv)?);
        let mut host_jsons = Vec::new();

        for line in reader.lines() {
            const REQUIRED_FIELDS: usize = 4;
            let mut fields = MaybeUninit::<[&str; REQUIRED_FIELDS]>::uninit();
            let val = line?;
            let field_count = val.matches(',').count() + 1;

            if field_count != REQUIRED_FIELDS {
                let e = make_io_err!(
                    InvalidData,
                    "Invalid number of fields in argument \"{}\" (expected {}, got {})",
                    val,
                    REQUIRED_FIELDS,
                    field_count
                );
                return Err(e);
            }

            for (i, field) in val.split(',').enumerate() {
                unsafe {
                    (fields.as_mut_ptr() as *mut &str).add(i).write(field);
                }
            }

            let [hostname, path, pointer, field] = unsafe { fields.assume_init() };
            host_jsons.push(HostJson::new(hostname, path, pointer, field));
        }

        Self::from_jsons(&host_jsons)
    }

    /**
     * Get the name of given address' host, if any.
     *
     * Returns `Some(host)` if `addr` was found in a host's network, or `None` if no hosts were
     * found.
     */
    pub fn get_host(&self, addr: Ipv4Addr) -> Option<&str> {
        for host in self.hosts.iter() {
            let has_addr = host
                .subnets
                .iter()
                .find(|subnet| subnet.contains(&addr))
                .is_some();

            if has_addr {
                return Some(&host.name);
            }
        }

        None
    }
}

/**
 * Extract an array of values of type `T` from the fields specified in `host_json`.
 *
 * Returns `Ok(vals)` if the values were successfully parsed from the JSON, or an `io::Error`
 * otherwise.
 */
fn extract_from_json<T: str::FromStr>(host_json: &HostJson) -> io::Result<Vec<T>> {
    let raw_json = fs::read_to_string(&host_json.path)?;
    let json: serde_json::Value = serde_json::from_str(&raw_json)?;
    let prefixes = json
        .pointer(&host_json.array_pointer)
        .map(serde_json::Value::as_array)
        .flatten()
        .ok_or_else(|| {
            make_io_err!(
                InvalidData,
                "Invalid JSON format: {}",
                host_json.path.display()
            )
        })?;

    let mut subnets = Vec::new();

    for info in prefixes {
        if let Some(prefix) = info[&host_json.field].as_str() {
            let subnet = prefix
                .parse()
                .map_err(|_| make_io_err!(InvalidData, "Invalid Ipv4Net format: {}", prefix))?;
            subnets.push(subnet);
        }
    }

    Ok(subnets)
}
