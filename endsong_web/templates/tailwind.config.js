// npx tailwindcss -i base_style.css -o tailwind_style.css --watch

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./404.html"],
  theme: {
    extend: {},
  },
  plugins: [],
}

