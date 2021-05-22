# Cloudcheck

## About

This work-in-progress project checks IP addresses against known networks that are owned by major cloud hosting companies. The intended use case for this is to make it easier to report malicious activity hosted on cloud services.

## Usage

The usage for the command-line interface is as follows:

    cloudcheck [OPTIONS] --csv <CSV>

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
        -a, --addresses <ADDRESS>...    IP addresses to check
        -c, --csv <CSV>                 String specifying a path to a CSV with columns "HOSTNAME,PATH,POINTER,FIELD" where
                                        HOSTNAME is the name of the host (e.g., "Google Cloud"), PATH is the path to the
                                        JSON file containing the IP ranges, POINTER is a JSON pointer to an array of objects
                                        in the CSV file, and FIELD is the object field that contains the IP address.
        -f, --files <INPUT_FILE>...     Files to check, with one IP address per line

## Examples

The file `examples/scan-sshd-logs.sh` contains a POSIX shell script that scans `/var/log` files for invalid user disconnections logged by `sshd` and prints the corresponding IPv4 address if it's hosted by Google Cloud or Amazon Web Services.

## IP Address Range Sources

* Google Cloud: https://www.gstatic.com/ipranges/cloud.json
* AWS: https://ip-ranges.amazonaws.com/ip-ranges.json
