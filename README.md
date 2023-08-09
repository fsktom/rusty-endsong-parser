# rusty-endsong-parser

Better, more performant version of <https://github.com/fsktom/endsong-parser-python> written in Rust

[Rust cheat sheet](https://docs.google.com/document/d/1kQidzAlbqapu-WZTuw4Djik0uTqMZYyiMXTM9F21Dz4)

## Pages

- [ideas](stuff/ideas.md)
- [notes](stuff/notes.md)
- [Spotify PDF explaining the `endsong.json` file](stuff/ReadMeFirst_ExtendedStreamingHistory.pdf)

## Assumptions made in this program

- all streams happened in one time zone and that timezone is your
computer's local time zone
  - dealing with changing time zones is too much of a hassle
  - Spotify saves the timestamp of your stream as an UTC date
that is then transformed to your local time zone
- timestamps are unique
  - i.e. no two entries share the same timestamp (if they do, only
  the first one is taken and the other are discared)
    - this has happened a lot to my earlier entries
- there are no two different artists with the same name
  - this is obviously not true, but the only way (that I can think of)
  to work around that is use the Spotify API
- there are no two different albums from the same artist
with the same name
  - impossible to check without Spotify API
