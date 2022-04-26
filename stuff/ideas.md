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
