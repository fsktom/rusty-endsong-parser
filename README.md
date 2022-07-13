# rusty-endsong-parser

Better, more performant version of https://github.com/Filip-Tomasko/endsong-parser-python written in Rust

[Rust cheat sheet](https://docs.google.com/document/d/1kQidzAlbqapu-WZTuw4Djik0uTqMZYyiMXTM9F21Dz4)

## Pages

- [ideas](stuff/ideas.md)
- [notes](stuff/notes.md)
- [Spotify PDF explaining the `endsong.json` file](stuff/ReadMeFirst_ExtendedStreamingHistory.pdf)

## Assumptions made in this program

- there are no two different artists with the same name
  - this is obviously not true, but the only way (that I can think of)
  to work around that is use the Spotify API
- there are no two different albums from the same artist
with the same name
  - impossible to check without Spotify API
