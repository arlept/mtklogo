
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![License: APACHE](https://img.shields.io/badge/license-APACHE-blue.svg)](LICENSE)

## About

mtklogo is a Command Line Interface with a thin reusable library which lets you hack
an MTK "LOGO" image. The main (and probably only) use case is replacing
your vendor logo with a custom one when your MTK-based phone boots!

This program is able to unpack, and subsequently repack all logo images (including
the big logo, the phone charging animation, etc. provided you can tell - or guess - the dimension
of these images).

**WARNING**: replacing the vendor's logo image may break your device.

Do backups before proceeding.

I can't be responsible for any damage caused to your device, use this material at your own risk.


### Why did I really make this?

Just for fun, to replace my phone's boot logo and to explore coding in Rust.
And by the way the [other](https://github.com/bgcngm/mtk-tools)
[tools](https://github.com/rom1nux/mtkimg) I found to do this sort of thing did not work for my phone, because
it has a different image format.

### How can this tool help me replacing the logos?

This tool does not change anything on your Android device.

It just lets you grab PNG images from a logo binary image that you
had previously extracted, and then rebuild another
logo binary image with a different set of PNG images.

You have to extract and flash again your `logo.bin` partition for the changes to take effect.
There are many ways to achieve this:

* Using the SP Flash Tool program. This does not require you to root or alter the system of your device.
It will probably void your warranty, though.is ppor
* Using backup and restore of [TWRP](https://twrp.me).
That alternative recovery program can fairly well backup and restore your `logo.bin` partition.
* On a rooted device, you can tinker with the `/dev/...` blocks...

## Program Usage

Type `mtklogo help` to list all commands.

Type `mtklogo help <command>` to get help for a specific command.

### Main usage

* You manage to get the `logo.bin` image corresponding to your device.
* You use the `mtklogo unpack` command to extract logos to some directory.
This gives you a set of files with a naming convention. Extracted logos
have the ".png" extension. Non-extracted logos have the ".z" extension.
The file name indicates the index and the encoding of the logo.
For example, you want to replace only the big boot logo on a recent phone:
```bash
mkdir /tmp/my-logos
mtklogo unpack logo.bin -o /tmp/my-logos --mode BgraBig --slots 0
```
* You can edit any ".png" file and replace it with another image, provided
it has the same dimension (same width and same height). You **must not** change
the file name. The file name contains indications necessary to the repack command.
```bash
gimp /tmp/my-logos/logo_000_bgrabe.png # do your changes to the image.
```
* You use the `mtklogo repack` command to rebuild a modified `logo.bin` logo image from
the set of logos which were extracted. Basically you pass `logos_*` as the list of files to rebuild.
```bash
mtklogo repack -o mylogo.bin /tmp/my-logos/logo_*
```
* You then manage to replace ("flash") the logo image with the repacked one `my-logo.bin`.

WARNING: It's tempting to replace poor, and incidentally lightweight, vendor's logo with heavy colorful ones.
For instance, my phones have an 8MB partition for the logo image, so they can cope with
one big boot logo. You must make sure that the repacked logo.bin size does not exceed
the logo partition size. 

### `unpack` command

`unpack` reads a logo binary image, then attempts to extract all images as `.png` files.

The MTK logo binary image does not contain any information about the images themselves: you don't know
their dimensions, or their image format. The only information one surely knows is their length in bytes.
We need a configuration file, which gives a list of possible dimensions as width (`w`) and length(`l`), for a  `color_model` (how colors are encoded, how many bytes do we need to represent a single pixel).

This [configuration](cli/resources/bin/mtklogo.yaml) file is loaded from the following location (in this order):

* specified by user `mtklogo unpack -c /path/to/my/configuration.yaml`
* in user's home/config directory `~/.config/mtklogo.yaml`
* as a sibling of the program itself `$(dirname $(which mtklogo))/mtklogo.yaml`
* as a global configuration file `/etc/mtklogo.yaml`

The default configuration gives a list of common dimensions for "big" logos, assuming the images are encoded in 16 bits rgb.
It contains two example profiles that you can adapt to your own device.
Feel free to edit that configuration file. 

Unpack examples:

Extracting all logos to current directory:

```bash
mtklogo unpack logo.bin
```

Extracting all logos to `/tmp/logos`, assuming 32 bits Bgra, big endian, images:

```bash
mtklogo unpack logo.bin -o /tmp/logos/ --mode bgrabe
```

Extracting only first two logos to `/tmp/logos`, using a specified custom profile:

```bash
mtklogo unpack logo.bin -o /tmp/logos/ --profile thl5000 --slots 0,1
```

### `repack` command

`repack` does the opposite of `unpack` it takes a set of files and creates a logo image from them.

The set of files must obey the naming convention used by "unpack", which is the following one:

```bash
logo_000_bgrabe.png
^    ^   ^      ^__ ".z" or ".png"
|    |   |_________ image encoding
|    |_____________ logo index (3 digits)
|__________________ always starts with "logo_"
```

".png" files are first encoded to device-specific format, then zipped. ".z" files are taken as-is.

Edge case: the 'repack' command just takes the logo images in the order specified by the logo index.
It won't complain if there is a missing, or duplicate index.

Repack example:

Repacks all logos extracted into /tmp/logos as mylogo.bin:

```bash
mtklogo repack -o mylogo.bin /tmp/logos/logo_*
```

### `explore` command

`explore` is useful when you don't know the dimension and the encoding of your images.
It will do something a bit overkill: try to export all images in every supported encoding!
You can then quickly with an image viewer which format is best.

By trial and errors, you will be able to narrow the dimension and color mode of all images,
and then create your own profile in the configuration file.

This command requires you to provide an expected "width". You're probably able
to guess this width for the big boot logo which is most of the time (if not all)
a fullscreen image at slot 0; just use the maximum screen width of your phone, you
just have to look at your phone specification's sheet.

Example: extracts boot logo of a thl5000 phone to /tmp, [knowing](https://www.devicespecifications.com/en/model/446a2c93) this device has a 1080 x 1920 screen:

```bash
mtklogo explore thl5000.bin --slots 0 --width 1080 -o /tmp
```

Now try viewing all /tmp/explore_logo_000_xxx.png files. The "best" image
will tell you what is this device image encoding (spoiler: it's rgba565 little endian).

### guess

`guess` does a (not so) trivial computation : given a size of N bytes, what can be the image dimension
in the different image formats?

Warning: there may be more solutions than you imagine. If you're nasty and try this command
with a high prime number, it might never return!

Example: if you're not confident that 3194880 = 1024 height * 780 width * 4 bytes_per_pixel (because it's RGBA),
you can see other (unlikely) solutions: 

```bash
mtklogo guess --size 3194880
... snip
if 4 bytes per pixel (modes: [Rgba(Big), Rgba(Little), Bgra(Big), Bgra(Little)]), 3194880 bytes is 798720 pixels and has following divisors: 2^12 * 3 * 5 * 13.
... snip
It could be 1024 x 780 ... 798720 = (2^10) * 780
It could be 3072 x 260 ... 798720 = (2^10 * 3) * 260
It could be 15360 x 52 ... 798720 = (2^10 * 3 * 5) * 52
It could be 199680 x 4 ... 798720 = (2^10 * 3 * 5 * 13) * 4
... snip
```


## Build and Install instructions

### Installing on your machine

This is the easiest way to build and run the tool.

This is built using rust 1.31.1.
Once you've the rust and cargo [tool chain](https://rustup.rs/), just install it:

```bash
# install it using cargo
cd cli
cargo install --path .
# copy the sample configuration to your home directory
cp resources/bin/mtklogo.yaml ~/.config 
```

It was tested on Debian Buster and Windows 10 and it probably works on any other Rust-enabled system.

## Compiling for an Android system

Disclaimer: I'm by no mean an Android expert, I'm just giving example 
instructions to compile on a Debian-based system, targetting an arm64-based Android system.
If your configuration is different, you will have to do your own research.

You will need tools from the [Android NDK](https://developer.android.com/ndk/downloads/)
because this program includes some C code ([miniz](https://code.google.com/archive/p/miniz/))
as a dependency, which needs to be rebuilt.
Oh, and by the way you hopefully already have Python... gosh what a language mess...

```bash
# get and install the NDK toolchain for arm64 into /opt/android/ndk/arm64
curl -O https://dl.google.com/android/repository/android-ndk-r17c-linux-x86_64.zip
unzip android-ndk-r17c-linux-x86_64.zip
mv android-ndk-r17c /opt/android
/opt/android/android-ndk-r17c/build/tools/make_standalone_toolchain.py \
  --arch arm64 --install-dir /opt/android/ndk/arm64
```

Then add the Rust toolchain for the target architecture.

```bash
rustup target add aarch64-linux-android
```

Add the following to `~/.cargo/config`:

```toml
[target.aarch64-linux-android]
linker = "aarch64-linux-android-gcc"
```

Then build, making sure to have the arm64 compiler toolchain in your PATH:

```bash
NDK=/opt/android/ndk/arm64
export PATH=$NDK/bin:$PATH
cargo build --release --target=aarch64-linux-android
# optional : make a smaller binary (removes symbols)
$NDK/aarch64-linux-android/bin/strip target/aarch64-linux-android/release/mtklogo
```

Time to test!

```bash
# let's push the binary
adb push target/aarch64-linux-android/release/mtklogo \
  resources/bin/mtklogo.yaml /data/local

# let's do something with it on Android
adb shell
su # become root
cd /data/local

# get the logo image of this phone.
cat /dev/block/platform/mtk-msdc.0/11230000.msdc0/by-name/logo >logo.bin

# unpack all images.
./mtklogo unpack logo.bin --profile lenovo_p1ma40

# download some image - anything returned by an image search "720 x 1280 in png format" is OK.
curl -O http://www.qiura.net/web/wallpapers/sorey-tales-of-zestiria-the-x/720x1280.png

# replace the logo 0 with that image we downloaded.
cat 720x1280.png > logo_000_bgrabe.png

# repack the logo binary
./mtklogo repack --output mylogo.bin logo_*

# *do* check that repacked file is smaller than original logo !
du -h logo.bin mylogo.bin

# now let's "flash" the new logo image, the quick way.
# reminder: it's just a test, I don't recommend in any way to do this!
cat mylogo.bin > /dev/block/platform/mtk-msdc.0/11230000.msdc0/by-name/logo

# you can now reboot the phone et voila!
```

## Future Improvements

Things which need to be improved:

* User messages
  * Have a verbose and a silent flag.
  * Ability to turn off colored output (command line arg AND compilation feature)
* Binary distribution. I understand some people may just want to change their
  logo and are not interested in building from source, so releasing windows executable
  is probably welcome. 
  * try making the executable smaller, it's overweight for its purpose.
  

## Support

This program is personal work and I have few time to do support, so it
probably won't be actively maintained. Please do note I will be of no help for guidance with
specific devices, android advices for rooting, flashing, etc.

Feel free to open issues, fork it, or submit merge requests.

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.
