# Headset profiles

One file per headset, in the same `key: value` syntax as the recipes (`recipes/README.md`): a line starting with `#` is a comment, blank lines are skipped, and one pair of double quotes around a value is dropped. The folder ships with the app, versioned with the code, and is read by `crates/engine/src/headsets.rs`.

Files are named `<id>-<slug>.headset`, for example `quest-2.headset`. The loader reads every `.headset` file in this folder and ignores other files, this README included. A file that does not parse or validate is an error naming the file, and any key outside the list below is refused.

## Keys

Every file must give these keys, in this order when one is missing:

| Key | Meaning | Allowed value |
|---|---|---|
| `name` | the headset's name for people | non-empty text |
| `usb-id` | the USB identification | `vendor:product`, four hex digits each, for example `2833:5013` |
| `adb-product` | the value `adb devices` reports as the product | non-empty text without spaces |
| `adb-model` | the value `adb devices` reports as the model | non-empty text without spaces |
| `panel-width` | the panel resolution per eye, width in pixels | a whole number above 0 |
| `panel-height` | the panel resolution per eye, height in pixels | a whole number above 0 |
| `refresh-rates` | the panel's refresh rates in Hz, low to high | whole numbers above 0, separated by `,`, each once |
| `connections` | the connection types the headset supports | `usb` or `wifi`, separated by `,`, each once |
| `client-app` | the client app's Android package | non-empty text without spaces |
| `client-app-name` | the client app's name for people | non-empty text |
| `stream-width` | the recommended stream size per eye, width in pixels | a whole number above 0 |
| `stream-height` | the recommended stream size per eye, height in pixels | a whole number above 0 |
| `stream-fps` | the recommended stream frame rate | a whole number above 0 |
| `frame-generation` | the frame-generation multiplier | a whole number from 1 to 8; 1 means no frame generation |
| `stream-connection` | the recommended connection | one of the values in `connections` |

These keys are optional:

| Key | Meaning | Allowed value |
|---|---|---|
| `field-of-view` | the maker's field of view in degrees | a number above 0 and at most 180 |
| `field-of-view-note` | where the field of view comes from, or why it is not given | non-empty text; required when `field-of-view` is present |

The engine returns the side-by-side stream size from the per-eye stream size: twice the stream width, by the stream height. For the Quest 2 that is 2880x1440.

When a headset is not in this folder, the engine returns safe defaults marked unknown: 1440x1440 per eye at 60 fps, with no frame generation (the multiplier is 1).

A complete example:

```text
# Example headset
name: Example headset
usb-id: 1234:5678
adb-product: example
adb-model: Example_1
panel-width: 2000
panel-height: 2000
refresh-rates: 72, 90
field-of-view: 100
field-of-view-note: the maker's specification sheet
connections: usb, wifi
client-app: org.example.client
client-app-name: Example Client
stream-width: 1600
stream-height: 1600
stream-fps: 72
frame-generation: 2
stream-connection: usb
```
