## Demo

### Use with GTK
![gtk](https://github.com/qingxiang-jia/ibus-cloud-pinyin/assets/5571586/4bec71f0-ca41-4928-b74d-83d42cfdad80)

### Use with kimpanel
![kimpanel](https://github.com/qingxiang-jia/ibus-cloud-pinyin/assets/5571586/73ae1973-dbb9-4c46-a391-8cdc358268ed)

## What is this?

This is a spin-off of the Full Cloud Pinyin [project](https://github.com/qingxiang-jia/full-cloud-pinyin). The idea is, since all communication with IBus is done via DBus, we don't need to rely on any IBus libraries to implement an input method (in this case, a cloud one). Instead, we can leverage on DBus libraries such as [zbus](https://github.com/dbus2/zbus). The benefit of this approach is:

1. No dependency on any non-Rust libraries (after all, making and maintaing C bindings is not fun).
2. The code is self-contained and you get a standalone binary for an input method. No packaging is needed and you can distribute the binary anywhere and it will work on any Linux that has IBus support.

However, there's some challenges:

1. Because it doesn't rely on any existing IBus libraries, you are building it yourselves. It's doable (since it's done in this project), but the intial learning curve is steep, especially on the handshake with IBus.
2. The IBus's DBus interface is actually not that well documented. I spent a lot of time testing what's the actual wire format. The good news is, if you build on top of this project, the essential ones have been implemented. But the bad news is, if you want to use a new IBus API, you need to start this trial-and-error process again. To make it worse, the infrastructure of debugging DBus is also quite lacking.

## Why this project?

The project is here because in Full Cloud Pinyin [project](https://github.com/qingxiang-jia/full-cloud-pinyin), I have switched gear and decided to focus on supporting only Fcitx5. It's a newer codebase and from my experience, it works better than IBus on both X11 and Wayland.

## People to thank
@[diwic](https://github.com/diwic) from dbus-rs and @[zeenix](https://github.com/zeenix) from zbus helped me overcome the initial hardship working with DBus.

## Prior arts
This project took inspiration from the [goibus](https://github.com/sarim/goibus) project, where it implements communications to IBus in pure Go.
