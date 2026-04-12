One failure moving our servers from deb12 to 13 was some packages moved to `contrib` repo.

After some time debugging our failed upgrade, turns out `cloud-init` thinks a missing pacakge is no big deal ¯\_(ツ)\_/¯

> cloud-init.log:2026-04-11 22:28:00,694 - apt.py\[DEBUG\]: The following packages were not found by APT so APT will not attempt to install them: \['geoipupdate'\]

tried to update our deployment following their [example](https://docs.cloud-init.io/en/latest/reference/examples.html#additional-apt-configuration-and-repositories)

```
apt:
  preserve_sources_list: true
  sources_list:
    deb $MIRROR $RELEASE main contrib
```

but that doesn't seem to do absolutely anything. I see that line mentioned in `/var/log/cloud-init.log` but nothing else happens because of it. Nada changes in `/etc/apt/**`. And cloud-init continue to ignore that package.

What's the right way to set the contrib repo? I don't want to add a full mirror url line since my provider image already have their local mirrors, so I would like to simply add a extra repo.

also tried the older(?) format, with no success

```
apt:
  sources:
    debian.sources:
      source: deb $MIRROR $RELEASE main contrib
      suites:
        - $RELEASE
        - $RELEASE-security
        - $RELEASE-updates
  update: true
  upgrade: true
```

edit: also tried

```
write_files:
  - path: /etc/apt/sources.list.d/99_contrib.sources
    content: |
      Types: deb
      URIs: mirror+file:///etc/apt/mirrors/debian.list
      Suites: trixie trixie-updates trixie-backports
      Components: contrib
      Signed-By: /usr/share/keyrings/debian-archive-keyring.gpg

      Types: deb
      URIs: mirror+file:///etc/apt/mirrors/debian-security.list
      Suites: trixie-security
      Components: contrib
      Signed-By: /usr/share/keyrings/debian-archive-keyring.gpg
```

and it shockingly did not work. If i save the same file and run `apt update` it does pick it up. if I just edit the cloud-init saved file with vim, and save, `apt update` get it. but if I just leave it as cloud init leaves, `apt update` completely ignores that file?!?! Am i going crazy now? same permission, ownership, everything.