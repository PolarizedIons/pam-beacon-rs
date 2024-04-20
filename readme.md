# PamBeaconRS

Place a file at `/home/username/.pambeacon` containing a list of mac addresses
of beacons that should allow access, one per line.

Next, build and place the resulting `libpambeacon.so` file somewhere accessible
to the pam service.

Add the following line to the pam-services that should use it:
```
auth required <path...to...libpambeaconrs.so>
```

Test with `pamtester`:

```shell
pamtester <servicehere> $USER authenticate
```

Inspired by https://github.com/muesli/pam-beacon