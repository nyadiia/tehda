# tehda

tehda ([ᴛᴇʜ-dah /ˈtɛ.dɑ/ or /teh.da/; Finnish for "to do, perform, execute"][tehda-wikt])
is a launcher/menu program, like [`dmenu`][dmenu], [`rofi`][rofi], or
[`wofi`][wofi], written in Rust. It runs on [wlroots][wlroots]-based
[Wayland][wayland] compositors.

## Build

```sh
cargo build
```

## Config

Config is contained within a `tehda.yaml` file within one of these locations:

- `$XDG_CONFIG_HOME/tehda/tehda.yaml`
- `$HOME/.config/tehda/tehda.yaml`
- `~/.config/tehda/tehda.yaml`
  (which should all refer to the same file in most cases).

If you would like to create a config from the defaults, run `tehda --dump-config`,
which will print the config to standard output.

Documentation on these config variables is TODO.

## License

GPL-3.

[tehda-wikt]: https://en.wiktionary.org/wiki/tehd%C3%A4#Finnish
[dmenu]: https://tools.suckless.org/dmenu/
[rofi]: https://github.com/davatorium/rofi
[wofi]: https://hg.sr.ht/~scoopta/wofi
[wlroots]: https://github.com/swaywm/wlroots
[wayland]: https://wayland.freedesktop.org/
