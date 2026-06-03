/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        mono: [
          "ui-monospace",
          "SFMono-Regular",
          "Menlo",
          "Consolas",
          "monospace",
        ],
      },
      colors: {
        ink: "#0e1116",
        panel: "#161b22",
        line: "#222831",
        muted: "#6b7280",
        accent: "#5eb8ff",
        pass: "#3fb950",
        warn: "#d29922",
        fail: "#f85149",
        na: "#6b7280",
      },
    },
  },
  plugins: [],
};
