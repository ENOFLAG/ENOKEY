# ENOKEY

[![Build Status](https://travis-ci.org/ENOFLAG/ENOKEY.svg?branch=master)](https://travis-ci.org/ENOFLAG/ENOKEY)
[![](https://tokei.rs/b1/github/ENOFLAG/ENOKEY)](https://github.com/ENOFLAG/ENOKEY)


This is the ENOKEY project for handling ssh keys. It contains the following tools:

* enokey - Collect, Generate and Distribute an authorized_keys from user input over a webservice


# Usage
```
$ ./enokey --help
Usage: target/debug/enokey [options]

Options:
    -s, --storage ENOKEYS.storage
                        The output ENOKEYS storage file
    -o, --authorized_keys authorized_keys
                        The output authorized_keys file
    -n, --dry-run       Do not write to cfg file
    -h, --help          Print this help menu
```

# Configuration Files

The configuration files declare which keys are to be included into the output file of authorized keys. There are three commands supported to include keys:

* provider:username - use the SSH keys stored on a well-known provider (currently GitHub and Gitlab)
* pubkey:actual-key - just include this key, formatted like usual in authorized key files
* include:file - include another config file

An example is given in [example-config](example-config):

```
# Comments and empty lines are ignored

# Use the scheme <provier>:<username> to scrape the keys there
github:torvalds
gitlab:stallman

# Just add a single key
pubkey:ssh-rsa AAAAB3NzaC1yc2EAAAABIwAAAQEAyiWjcfZ2QN2aigzjvvAMmLJ70lXbN92IGyuY3tQOrA162Jtn6OSDIUNcR3q8as6LrGlX2LJZAygndB59Mb12Zddv2nB/UuanD3x1R47fMA2iliMjanQSjDbtEgtDi6u/cArvb1PA4P9FUjxUx7RdNKd4RuYrFyOVMmPpbqD7x5QBHZT7y43mrHCYAoYEoOZdVrXcMVxnit2iN9oA3f+h5GmVRgciIXxgqBbdvRmADBrR9jkeQGFPOVdRfVGLxpFMeM+abm3+JmJIMxneiLcO2hxx+47MvMuALrLzoSztkks+HeiRkiv1bXOuXdUMFcrHNuwaJ/f5lqJtp8fdJ1+riQ== randomuser@machine

# Include a config file
#include:another-config
```

This is converted to the following:

```
###
# Config: example-config
###
# Keys from torvalds@github
ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQCoQ9S7V+CufAgwoehnf2TqsJ9LTsu8pUA3FgpS2mdVwcMcTs++8P5sQcXHLtDmNLpWN4k7NQgxaY1oXy5e25x/4VhXaJXWEt3luSw+Phv/PB2+aGLvqCUirsLTAD2r7ieMhd/pcVf/HlhNUQgnO1mupdbDyqZoGD/uCcJiYav8i/V7nJWJouHA8yq31XS2yqXp9m3VC7UZZHzUsVJA9Us5YqF0hKYeaGruIHR2bwoDF9ZFMss5t6/pzxMljU/ccYwvvRDdI7WX4o4+zLuZ6RWvsU6LGbbb0pQdB72tlV41fSefwFsk4JRdKbyV3Xjf25pV4IXOTcqhy+4JTB/jXxrF torvalds@github (1)
# Keys from stallman@gitlab
ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQCfu9PhGt4+RcGdxpHnbu129OBNhXZhUEpVVWTXeOwq68+CgdE25hjW3qyJkNDe/3uno9ogCg/FXa083r6bQt5YJU65o22yYBFXe+m1OcT4Uw56nkcT9hjJqJxHg1+DWKlheNhth5VQOVueyN8SPTKU6ezelcpWiOXfRBu5DOhGKkooT98f4HiujmrSkCD/1WIjAA4m0rBYF8PmXLW0qFiiw4mPxAAVXRu+lF6tPTqT9gSwUTgKcJ/LTd79caU0H0jsqsF9S/+s7/dMqR3TRGVnAUUQJlKyizA9mg2mJ91bBVGVSE/Aiyo3788vZekBqM7mWI74ZgePwf+EgT7yRlf1  stallman@gitlab (1)
# Keys from pubkey
ssh-rsa AAAAB3NzaC1yc2EAAAABIwAAAQEAyiWjcfZ2QN2aigzjvvAMmLJ70lXbN92IGyuY3tQOrA162Jtn6OSDIUNcR3q8as6LrGlX2LJZAygndB59Mb12Zddv2nB/UuanD3x1R47fMA2iliMjanQSjDbtEgtDi6u/cArvb1PA4P9FUjxUx7RdNKd4RuYrFyOVMmPpbqD7x5QBHZT7y43mrHCYAoYEoOZdVrXcMVxnit2iN9oA3f+h5GmVRgciIXxgqBbdvRmADBrR9jkeQGFPOVdRfVGLxpFMeM+abm3+JmJIMxneiLcO2hxx+47MvMuALrLzoSztkks+HeiRkiv1bXOuXdUMFcrHNuwaJ/f5lqJtp8fdJ1+riQ== randomuser@machine pubkey (1)
```
