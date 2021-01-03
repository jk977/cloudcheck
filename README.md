# SSHD Failcheck

## About

This work-in-progress project analyzes `/var/log` files for SSHD authentication failures that are owned by major hosting companies (currently Amazon and Google). In the future, it will generate separate logs for each company to allow these IP addresses to be automatically reported to the company for abuse.

## IP Address Range Sources

* Google Cloud: https://www.gstatic.com/ipranges/cloud.json
* AWS: https://ip-ranges.amazonaws.com/ip-ranges.json
