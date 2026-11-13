# Download pins

`pins` is the repository's source table for binary fetches. Each non-comment line is `key.field: value`; `#` comments occupy whole lines and inline comments are not allowed. Fetch keys declare `fetch: auto` or `fetch: manual`. Automatic keys include `version`, `url`, `size`, `sha256` and `license`. A manual key includes `page` and `license`. `crate.*` and `npm.*` lines are version lists, not fetch keys.

Unknown source metadata stays `TBD-at-pin`; this is not a usable checksum. An automatic row may keep a TBD URL or size only while its sha is TBD. `pin.sh` refuses a TBD URL and names its page so the owner can fill the source before pinning. Pages not available from the approved release metadata are also marked TBD for owner follow-up.

Run `packaging/pin.sh <key>` to fetch one automatic asset into the user cache, verify its size/checksum and (when GitHub supplies it) release digest, and record the SHA, discovered size, and pin date. It never extracts or runs the downloaded file. It prints a diff-stat reminder for review. A known hexadecimal SHA requires `--check`; `--check` verifies without writing. `--dry-run` prints the would-be pin lines without editing. `--file` uses a local file instead of a download for controlled tests. Local test fixtures may enable `PIN_ALLOW_FILE=1`; normal operation accepts only HTTPS URLs. `--all-tbd` attempts every automatic row whose SHA is still TBD.

Exit codes: `0` success, `2` bad usage/pins/key, `3` unavailable or disallowed source, `4` integrity mismatch, `5` already pinned. Every exit emits a stable `pin:` token and fix hint. Network and GitHub CLI failures are retryable; integrity mismatches are not.

The application fetches files for the user's own installation. The repository does not redistribute third-party binaries. `flat3d-link.service` leaves `KillMode` unset, so systemd's default `control-group` applies.

## Tools and the udev rule

`build-tools.sh` builds one tool, adb, from Android's open source (the `android-tools` pin) into `dist/tools/` with its non-system libraries in `dist/tools/lib/`. Flat to 3D ships no internet tunnel and no relay for one (ruling 42), so `PAYLOAD_TOOLS` is `adb` only.

`flat3d-quest.rules` is Flat to 3D's own udev rule, so a Quest works without Quest Tether (ruling 42, SPEC M28). It is one line that gives the logged-in user access to Meta USB devices (vendor `2833`, `TAG+="uaccess"`). The engine embeds it. `flat3d-inject install` writes a copy to `~/.local/share/flat3d/flat3d-quest.rules` and then, with one sudo prompt, installs it as `/etc/udev/rules.d/70-flat3d-quest.rules` (before `73-seat-late.rules`, which applies the tag), reloads the rules and re-triggers Meta USB devices. It skips that step when the file is already there. A refused or failed prompt is a warning in the install summary, not an install failure. `flat3d-inject uninstall` removes the file with one sudo prompt when it is there.
