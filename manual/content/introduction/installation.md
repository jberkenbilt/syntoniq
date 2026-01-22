+++
title = "Installation"
weight = 10
sort_by = "weight"
+++

<!-- Note: the top-level README.md has a deep link to this section. Update it if the file is moved. -->

Download a release from the [GitHub Releases page](https://github.com/jberkenbilt/syntoniq/releases). There you will find the following:

* Linux amd64 (x86_64) and arm64 (aarch64) with and without Csound
* MacOS universal binaries (supporting both Intel and Apple Silicon) with and without Csound
* Windows x86_64 (64-bit only) with and without Csound

See also [Verifying Releases](#verifying-releases).

You must download and install [Csound](https://csound.com) to use the Csound versions. The binary distributions are built with Csound version 6.18.1 but will likely switch to Csound 7 sometime after it is out of beta. The non-Csound versions include a `syntoniq-kbd` application that only works as a MIDI device. On Linux and Mac, it does not require additional software beyond something that can play MIDI.

On Windows, to use `syntoniq-kbd` as a MIDI device, you need loopmidi:
* Install https://www.tobias-erichsen.de/software/loopmidi.html
* Create a loop port called `syntoniq-loop`; the application expects a device by this name to exist

Linux and Mac distributions are compressed tarballs. Windows distributions are zip files. They contain only the binaries for `syntoniq` and `syntoniq-kbd`. You just need to put these somewhere in your path or directly execute them. For the Csound versions, make sure they can find the Csound libraries. Using the default installation methods for Csound, this should work automatically. For Windows, be sure Csound is in your path. The installer offers to do this for you when you install, though you will need to restart your shell for it to take effect.

## MacOS Notes

On the Mac, the OS will complain about the downloaded binaries since they are not signed. You can fix from the shell with
```sh
xattr -d com.apple.quarantine syntoniq syntoniq-kbd
```

You can also follow the process with the UI:
* Run the executable. You'll have to repeat this for each executable.
* You will get a pop-up dialog telling you "syntoniq" (or "syntoniq-kbd") was "Not Opened"
* There's a question mark hiding in the upper-left corner. Click it.
* This opens a page explaining about the issue. If you trust the downloads, you can find the "Open Privacy & Security settings for me" link. Click that.
* Scroll down to "Security" where it says "syntoniq" (or "syntoniq-kbd") was blocked to protect your Mac.
* Click "Allow Anyway."
* Run the executable again. This time, you will have "Open Anyway" as a choice. Once you click that, you're good to go until you replace the binary.

# Building from Source

Please see the [top-level README.md](https://github.com/jberkenbilt/syntoniq/blob/main/README.md) in the source repository for instructions.

# Verifying Releases

The `syntoniq-<version>.sha256` file contains sha256 checksums of all release assets. That file is also clear-signed with GPG using [this key](https://q.ql.org/pubkey.asc), with fingerprint `C2C96B10011FE009E6D1DF828A75D10998012C7E`. They are also signed with Cosign. You can verify with
```
cosign verify-blob syntoniq-x.y.z.sha256 --bundle syntoniq-x.y.z.sha256.sigstore \
   --certificate-identity=ejb@ql.org \
   --certificate-oidc-issuer=https://github.com/login/oauth
```
