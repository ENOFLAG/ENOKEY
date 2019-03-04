# ENOKEY

[![Build Status](https://travis-ci.org/ENOFLAG/ENOKEY.svg?branch=master)](https://travis-ci.org/ENOFLAG/ENOKEY)
[![](https://tokei.rs/b1/github/ENOFLAG/ENOKEY)](https://github.com/ENOFLAG/ENOKEY)


# Docker

ENOKEY is configured with environment variables. Here is an example using docker-compose.yml:
```
version: '3'

services:
  enokey:
    build: enoflag/enokey
    volumes:
      - ./data:/enokey/data
    restart: on-failure
    ports:
      - "80:8000"
    environment:
      - ROCKET_PORT=8000
      - ROCKET_ENV=production
      - ROCKET_LOG=normal
      - ROCKET_SECRET_KEY=whs/vijJnEoWN9Xgf25oJDn2yUtvsNuhm0eMNxZe6CI=
      - ADMIN_SERVERS=root@very.import.server:8022
      - ADMIN_PSK=HIGHLYSECRET
      - USER_SERVERS=root@boring.server
      - USER_PSK=NOTSOSECRET
      - RUST_BACKTRACE=1
```