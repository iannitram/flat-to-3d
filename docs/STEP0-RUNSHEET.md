# Step 0 run sheet: Spidy on Spider-Man Remastered, Quest 2, Z13

Goal: prove the whole chain works on this machine before porting anything. That's the Z13 under Proton, WiVRn over a USB cable, the Quest 2, and Spidy's **GAME SCREEN 3D** mode (the big 3D TV).

Do this in **Desktop Mode**. WiVRn's window and the scripts need the desktop.

Spidy stays on its pinned v0.3.8 for this step. Upstream's v0.7.0 Proton + WiVRn docs give the same launch options, so the run sheet is unchanged in substance; the decision and what moved are in the [version note](spiderman2/spidy-core-map.md#version-note-spidyvr-v038-and-v070-issue-38).

## One-time setup

1. **You:** In Steam, right-click Spider-Man Remastered → Properties → General. Set the compatibility tool to **Proton Experimental** (the build upstream tested with). Then Launch Options. Run `flatpak info io.github.wivrn.wivrn` in a terminal first: if WiVRn is a `--user` install, paste the line below; if it's system-wide, use `/var/lib/flatpak/app/io.github.wivrn.wivrn` in place of the path. Paste exactly, with `<your home>` replaced by your full home path (run `echo $HOME`):

   ~~~
   PRESSURE_VESSEL_IMPORT_OPENXR_1_RUNTIMES=1 PRESSURE_VESSEL_FILESYSTEMS_RW=<your home>/.local/share/flatpak/app/io.github.wivrn.wivrn STEAM_COMPAT_LAUNCHER_SERVICE=proton %command%
   ~~~

   These let the game see WiVRn and let Spidy run inside the game's Proton session. The game itself is unchanged. Upstream says to write the full home path.
2. **Script:** With the Quest plugged in, run `adb install` for the WiVRn APK. Its package name is `org.meumeu.wivrn.github`.
3. **You:** In the headset, use a **stationary** (seated) boundary. WiVRn pauses the game when you step outside it.

## Each run, in this order

The order matters: Proton connects to VR only when the game starts, so WiVRn must be running first.

1. **You:** plug the Quest 2 into the Z13 by USB and put it on. Accept "Allow USB debugging" if asked.
2. **Script:** start the WiVRn server, then connect the headset over the cable:
   ~~~
   adb reverse tcp:9757 tcp:9757
   adb shell am start -a android.intent.action.VIEW -d "wivrn+tcp://localhost" org.meumeu.wivrn.github
   ~~~
   The headset shows WiVRn's connected screen. If the picture stutters, turn the Quest's Wi-Fi off; WiVRn's docs say a headset also on Wi-Fi can fight the cable.
3. **You:** start Spider-Man Remastered from Steam. **First time only**, in the game's settings: frame generation **off**, vertical sync **off**, Ray-Traced Reflections **off**, Aspect Ratio **Auto**. Then wait at the main menu.
4. **Script:** run `bash tools/spidy-wivrn.sh` from the Spidy folder. The wrapper only sets the WiVRn runtime and hands off to Spidy's Linux launcher, which warns about start order or missing launch options before it attaches.
5. **You:** click the game window once and load your save with mouse or keyboard. VR starts when gameplay does, in full head-tracked VR by default.
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
| `-51` with the start order right | The game's container can't read WiVRn's manifest: check the `PRESSURE_VESSEL_FILESYSTEMS_RW` path against `flatpak info`. |
| `-51` after a WiVRn update | Restart WiVRn, then the game. For a log, add `PROTON_LOG=1` for one run, then remove it (it slows the game). |
| "The game's Proton session was not found" | The launch option `STEAM_COMPAT_LAUNCHER_SERVICE=proton` is missing, or the game isn't at the menu. |
| "This game version is not supported" | Steam has a newer Remastered than 4.630.0.0. Spidy needs an update. |
| Menus ignore the Quest controllers | Known Spidy limit on Linux: use mouse/keyboard in menus, or Properties → Controller → Disable Steam Input. |
| Heavy scenes drop eye frames | Known limit when attaching late: Spidy can't enlarge render memory. Expected; note it. |

Record the result in `docs/spiderman2/plan.md` and, if it works, file a compatibility report upstream to SpidyVR (Quest 2 + Radeon 8060S + SteamOS + WiVRn 26.9).
