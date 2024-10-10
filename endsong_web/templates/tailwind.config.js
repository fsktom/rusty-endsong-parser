// npx tailwindcss -i base_style.css -o tailwind_style.css --watch

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./base.html",
    "./index.html",
    "./404.html",
    "./artists.html",
    "./artists_search.html",
    "./artist.html",
    "./artist_selection.html",
  ],
  theme: {
    extend: {},
  },
  plugins: [],
};
