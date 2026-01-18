# CI Setup

The CI build uses some artifacts that were created manually and uploaded to unlinked locations in syntoniq.cc.

We use Csound 6.18.1. If that changes, remember to update `manual/content/introduction/installation.md` and the top-level README.md.

# Mac Csound

Install Csound from the mac installer from csound.com

```sh
cd /Library/Frameworks
tar czvf /tmp/csound-6.18.1.mac.tar.gz CsoundLib64.framework
aws s3 cp /tmp/csound-6.18.1.mac.tar.gz s3://web-qbilt-org/external/syntoniq.cc/ci-resources/
```

# Windows Csound

Install Csound Windows MSI installer. The Windows binaries zip for 6.18.1 isn't usable since the includes contain `.in` files that weren't resolved.

```sh
cd "/c/Program Files"
zip -r /tmp/csound-6.18.1.windows.zip Csound6_x64
aws s3 cp /tmp/csound-6.18.1.windows.zip s3://web-qbilt-org/external/syntoniq.cc/ci-resources/
```
