/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        text: "#150303",
        background: "#fbfbfe",
        primary: "#AB152E",
        secondary: "#ff93a5",
        accent: "#ae001c",
        dark: "#0d0505",
      },
      fontFamily: {
        sans: ["Inter", "system-ui", "sans-serif"],
      },
      screens: {
        xs: "320px",
      },
    },
  },
  plugins: [],
};
