/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        forge: {
          bg: "#0b0e14",
          panel: "#131722",
          border: "#232a3b",
          accent: "#3b82f6",
          danger: "#ef4444",
          warn: "#f59e0b",
          safe: "#22c55e",
        },
      },
    },
  },
  plugins: [],
};
