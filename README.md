# rusty-endsong-parser

Better, more performant version of https://github.com/Filip-Tomasko/endsong-parser-python written in Rust

## Assumptions made in this program

- there are no two different artists with the same name
  - this is obviously not true, but the only way (that I can think of)
  to work around that is use the Spotify API
