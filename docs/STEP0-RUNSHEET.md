# Step 0 run sheet: Spidy on Spider-Man Remastered, Quest 2, Z13

Goal: prove the whole chain works on this machine before porting anything. That's the Z13 under Proton, WiVRn over a USB cable, the Quest 2, and Spidy's **GAME SCREEN 3D** mode (the big 3D TV).

Do this in **Desktop Mode**. WiVRn's window and the scripts need the desktop.

## Already set up (2026-10-18)

| Piece | Where |
|---|---|
| WiVRn server 26.9 (Flatpak, user install) | `flatpak run io.github.wivrn.wivrn` |
| WiVRn headset app 26.9 (APK, same version as the server) | `~/.local/share/flat-to-3d/apk/WiVRn-release.apk` |
| Spidy 0.3.8 release (official zip, sha256 `d7277564…dd31f`) | `~/.local/share/flat-to-3d/spidy-0.3.8/Spidy-0.3.8/` |
| Spider-Man Remastered (Steam 1817070) | internal drive, downloading |

## One-time setup

1. **Launch options (Ian, in Steam):** right-click Spider-Man Remastered → Properties → General → Launch Options. Paste exactly:

   ```
   PRESSURE_VESSEL_IMPORT_OPENXR_1_RUNTIMES=1 PRESSURE_VESSEL_FILESYSTEMS_RW=/home/deck/.local/share/flatpak/app/io.github.wivrn.wivrn STEAM_COMPAT_LAUNCHER_SERVICE=proton %command%
   ```

   These let the game see WiVRn and let Spidy run inside the game's Proton session. The game itself is unchanged.
2. **Headset app (Claude, with the Quest plugged in):** `adb install` the WiVRn APK. Its package name is `org.meumeu.wivrn.github`.
3. **Boundary (Ian, in the headset):** use a **stationary** (seated) boundary. WiVRn pauses the game when you step outside it.

## Each run, in this order

The order matters: Proton connects to VR only when the game starts, so WiVRn must be running first.

1. **Ian:** plug the Quest 2 into the Z13 by USB and put it on. Accept "Allow USB debugging" if asked.
2. **Claude:** start the WiVRn server, then connect the headset over the cable:
   ```
   adb reverse tcp:9757 tcp:9757
   adb shell am start -a android.intent.action.VIEW -d "wivrn+tcp://localhost" org.meumeu.wivrn.github
   ```
   The headset shows WiVRn's connected screen. If the picture stutters, turn the Quest's Wi-Fi off; WiVRn's docs say a headset also on Wi-Fi can fight the cable.
3. **Ian:** start Spider-Man Remastered from Steam. **First time only**, in the game's settings: frame generation **off**, vertical sync **off**, Ray-Traced Reflections **off**, Aspect Ratio **Auto**. Then wait at the main menu.
4. **Claude:** run `bash tools/spidy-wivrn.sh` from the Spidy folder. It checks the order and the launch options, then attaches.
5. **Ian:** click the game window once and load your save with mouse or keyboard. VR starts when gameplay does, in full head-tracked VR by default.
6. **Switch to the 3D TV:** pause → Settings → **SPIDY VR** tab → **VR VIEW: GAME SCREEN**, and **GAME SCREEN 3D** at the highest step. **GAME SCREEN SIZE** to taste.

## What to check (pass = step 0 done)

- [ ] The game shows on a big screen in the Quest 2 and plays with the controller.
- [ ] **Depth:** close one eye, then the other. Near things (Spider-Man, rooftops) jump sideways between eyes; far buildings barely move.
- [ ] HUD text sits flat on the screen and is readable.
- [ ] Smooth enough: 30+ fps. Note roughly what it feels like.
- [ ] Pausing turns the screen flat; unpausing brings the 3D back.

## If it fails

| Symptom | Likely cause |
|---|---|
| `No headset found (wineopenxr64 ... -51)` | Game started before WiVRn, or the launch options are missing. Quit the game and redo from step 2. |
| "The game's Proton session was not found" | The launch option `STEAM_COMPAT_LAUNCHER_SERVICE=proton` is missing, or the game isn't at the menu. |
| "This game version is not supported" | Steam has a newer Remastered than 4.630.0.0. Spidy needs an update. |
| Menus ignore the Quest controllers | Known Spidy limit on Linux: use mouse/keyboard in menus, or Properties → Controller → Disable Steam Input. |
| Heavy scenes drop eye frames | Known limit when attaching late: Spidy can't enlarge render memory. Expected; note it. |

Record the result in `docs/PORT-SPIDERMAN2.md` (status line) and, if it works, file a compatibility report upstream to SpidyVR (Quest 2 + Radeon 8060S + SteamOS + WiVRn 26.9).
