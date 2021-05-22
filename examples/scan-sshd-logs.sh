#!/bin/sh
#
# This script checks SSHD log entries in `/var/log` and prints any Google Cloud
# or Amazon Web Services IPv4 addresses that tried to authenticate as an invalid
# user.

cloudcheck() {
    # run the program via cargo, forwarding any function arguments as CLI args
    cargo run --release -q -- "$@"
}

die() {
    # print args to stdout, then exit
    echo "$@" >&2
    exit 1
}

example_dir="$(dirname "$0")"
data_dir="$example_dir/../data"

if [ ! -d "$data_dir" ]; then
    die "Data directory not found: $data_dir"
fi

sshd_ip_field=11

find /var/log -maxdepth 1 -type f -readable -exec grep --ignore-case sshd '{}' + \
    | grep 'Disconnected from invalid user' \
    | tr --squeeze-repeats ' ' \
    | cut --delimiter=' ' --fields=$sshd_ip_field \
    | cloudcheck --csv "$data_dir/default.csv"
