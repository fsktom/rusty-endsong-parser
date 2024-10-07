// npx tailwindcss -i base_style.css -o tailwind_style.css --watch

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./base.html",
    "./index.html",
    "./404.html",
    "./artists.html",
    "./artist.html",
  ],
  theme: {
    extend: {},
  },
  plugins: [],
};
