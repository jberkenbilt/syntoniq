# HexBoard Arduino

Prior to release 1.3 of the HexBoard firmware, it is necessary to build a modified firmware.

# Install Arduino

```
curl -fsSL https://raw.githubusercontent.com/arduino/arduino-cli/master/install.sh | BINDIR=~/.local/bin sh
```

Created ~/.arduino15, .local/bin/arduino-cli, ~/Arduino

# Build Firmware

In the HexBoard firmware repo, see .build.yml for authoritative setup. Build with `make` -- no arguments or targets required.

```sh
make
# On device, select Advanced -> Update Firmware. Device might be different from /dev/sda.
dev=/dev/sda
sudo mount -o uid=$(id -u) ${dev}1 /mnt/tmp/1 && \
    cp ~/Q/projects/hexboard/build/build.ino.uf2 /mnt/tmp/1 && \
    sync && \
    sudo eject $dev
```

# Watch Midi Events

```sh
# find port
aseqdump -l
aseqdump -p <client:port>
```

# Watch Serial Logs

Enable serial debugging from HexBoard menu

```sh
ls -l /dev/serial/by-id
# One of these
socat -u /dev/ttyACM0,raw,echo=0 STDOUT
screen /dev/ttyACM0 115200
```

# Example Delegated Control SysEx

```sh
# Enter Delegated Control
amidi -p hw:4,0,0 -S F0 7D 01 F7
# Watch key events
aseqdump -p 32:0
# Set an LED -- see comments in firmware or Syntoniq HexBoard driver
amidi -p hw:4,0,0 -S F0 7D 03 00 51 3F 7F 7F F7
# Exit Delegated Control
amidi -p hw:4,0,0 -S F0 7D 02 F7
```
