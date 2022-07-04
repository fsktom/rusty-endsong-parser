# Ideas

## Pure Theory

- Web
  - WASM?
    - https://github.com/rustwasm/wasm-bindgen
  - Web as GUI
    - static HTML or local server?
  - for plotting
  - Yew, Seed Web Frameworks?
- CLI arguments: `rep --no-duplicates endsong_0.json endsong_2.json`
  - e.g. `--no-duplicates` would prevent duplicate checking
    - I think just checking the hash of every file, putting into array and check if array has duplicates
    - and this option if you're absolutely sure there's no duplicates
  - and the relative paths to files as arguments without the `--`
    - or maybe to a whole directory of `endsong_x.json` files?  
- [Plotters](https://old.reddit.com/r/rust/comments/ude3lz/plotters_is_back/) for graphs?
- [some Rust libraries](https://old.reddit.com/r/rust/comments/uevmnx/what_crates_would_you_consider_essential/)
- [clap - CL argument parser](https://docs.rs/clap/latest/clap/)
- [GUI library egui](https://old.reddit.com/r/rust/comments/ugefgv/egui_018_released/)
- [inquire library for interactive terminal](https://docs.rs/inquire/latest/inquire/)
- [Tauri for UI with Javascript and backend with Rust!](https://youtu.be/-X8evddpu7M)
- way for interacting with it:
  - after running it with the files as arguments you type in commands that do stuff
  - like `help`, `print top artist "Powerwolf"` or similar
  - maybe interactive menu like [this](https://code.visualstudio.com/api/extension-guides/color-theme#create-a-new-color-theme) [one](https://code.visualstudio.com/assets/api/extension-guides/color-theme/yocode-colortheme.png)
  - with autocomplete for commands AND (most importantly) artists, albums and songs !!
  - maybe select commands with the interactive menu/options and there you can go back or forward and change parameters or leave them at default and at the end you type in the aspect
    - THIS and a shorthand commands for power users (faster and easier to the same command but with one thing changed)
- adjust for DST + time zone
  - e.g. for my listens adjust when DST was in Germany and save the relative time from the absolute Unix timestamp
  - for beginning: hardcode when CET changes to CEST for 2016-2022
