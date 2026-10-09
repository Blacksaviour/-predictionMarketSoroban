/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/pages/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/components/**/*.{js,ts,jsx,tsx,mdx}",
    "./src/app/**/*.{js,ts,jsx,tsx,mdx}",
  ],
  theme: {
    extend: {
      colors: {
        // Stellar brand colors
        stellar: {
          blue: "#00D3FE",
          purple: "#8A2BE2",
          dark: "#0E0E10",
          card: "#1A1A1F",
          border: "#333338",
        },
        // Theme tokens used across components
        background: "#0A0A0B",
        foreground: "#E5E7EB",
        card: "#1A1A1F",
        "card-foreground": "#E5E7EB",
        border: "#333338",
        primary: "#00D3FE",
        "primary-hover": "#00B8E6",
        secondary: "#8A2BE2",
        muted: "#2A2A33",
        "muted-foreground": "#9CA3AF",
        success: "#22C55E",
        error: "#EF4444",
        warning: "#F59E0B",
      },
      backgroundColor: {
        "stellar-blue": "#00D3FE",
        "stellar-dark": "#0E0E10",
      },
      textColor: {
        "stellar-blue": "#00D3FE",
        "stellar-dark": "#0E0E10",
        "stellar-foreground": "#E5E7EB",
      },
      borderColor: {
        "stellar-blue": "#00D3FE",
      },
    },
  },
  plugins: [],
};
