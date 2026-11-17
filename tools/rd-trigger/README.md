# rd-trigger

Asks a running program that has RenderDoc's capture layer loaded to capture a frame, with no key press. RenderDoc's own capture keys are F12 and Print Screen; on the Z13, F12 is also airplane mode and there is no Print Screen key.

Needs the host's RenderDoc 1.46 (`/usr/lib/librenderdoc.so`, as SteamOS ships it).

```bash
tools/rd-trigger/build.sh                 # builds tools/rd-trigger/rd-trigger
rd-trigger --delay 20                     # wait 20 s, then capture one frame
rd-trigger --frames 2 --wait 120          # capture 2 frames; give up after 120 s
```

It connects to the newest RenderDoc target on this machine, triggers the capture, prints the `.rdc` path, and exits 0. It exits 1 if no target is running or no capture arrives in `--wait` seconds.

To run it from a key instead of a timer, bind it in KDE: System Settings, Shortcuts, Add New, Command or Script, command `rd-trigger` (full path if it is not on `PATH`), then press the key you want (the Copilot key works).
