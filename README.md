
<div align="center">

<h1>Headlines [WIP]</h1>


A cross platform native GUI app built with Rust using [egui](https://github.com/emilk/egui). Uses newsapi.org as the source to fetch news articles.
</div>


![screenshot](./assets/thumb.png)
### This is a WIP and the current status can be found in [implementation status](#implementation-status)

## Video walkthrough

[Part A](https://youtu.be/NtUkr_z7l84)

[Part B](https://www.youtube.com/watch?v=SvFPdgGwzTQ)

## Implementation Status

- [X] Base UI
- [X] Integrate dark mode
- [X] Integrate real articles feed.
- [X] Config window for setting API_KEY
- [X] State persistance
- [X] Cross platform

## Setup

### Linux

Make sure `egui` [dependencies](https://github.com/emilk/egui#demo) are installed.
Once done, run `cargo run` on the terminal (you need to have the Rust toolchain installed).

### Windows

Works as usual using `cargo run`

### Web

Run `./setup_web.sh`

Launch with `./start_web.sh`

## Contributions

All kinds of contributions are welcome.

## License

MIT
