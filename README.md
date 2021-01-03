# SSHD Failcheck

## About

This work-in-progress project analyzes `/var/log` files for SSHD authentication failures by IP addresses that are owned by major hosting companies (currently Amazon and Google). In the future, it will aggregate logs for each company to allow the addresses to be automatically reported for abuse.

## IP Address Range Sources

* Google Cloud: https://www.gstatic.com/ipranges/cloud.json
* AWS: https://ip-ranges.amazonaws.com/ip-ranges.json
