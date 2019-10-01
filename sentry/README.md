Sentry
======
Sentry is the backend for oxide.  It includes the following components:

* Token generator, adherance to [Atlassian S2S Auth Spec](https://s2sauth.bitbucket.io/spec)
* Token validator through async Tokio server
* Password hashing
* Collision free unique identifier generator.

##Build requirement
* Initialize PEM private and public keys in DER format.

```
# Run shell script for directory and key scaffold.
$ sh gen-keys.sh
```

* Store `MASTER_ASAP_KEY` in ./warden.key file recommended 256-bit minimum.

* Store `SECRET_KEY` in ../sentry.env file, recommend 128-bit minimum.

## License
This library is licensed under Apache License, Version 2.0, (LICENSE-APACHE or
http://www.apache.org/licenses/LICENSE-2.0)
