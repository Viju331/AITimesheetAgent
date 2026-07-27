/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/**/*.{html,ts,scss}',
  ],
  // Prevent Tailwind from resetting Angular Material styles
  corePlugins: {
    preflight: false,
  },
  // Namespace Tailwind utilities to avoid conflicts with Angular Material
  important: false,
  theme: {
    extend: {
      colors: {
        primary: {
          50:  '#e8eaf6',
          100: '#c5cae9',
          200: '#9fa8da',
          300: '#7986cb',
          400: '#5c6bc0',
          500: '#3f51b5',
          600: '#3949ab',  // default primary
          700: '#303f9f',
          800: '#283593',
          900: '#1a237e',
        },
        accent: {
          400: '#ffa726',
          700: '#f57c00',  // default accent
        },
      },
      fontFamily: {
        sans: ["'Inter'", "'Segoe UI'", 'Arial', 'sans-serif'],
        mono: ["'JetBrains Mono'", "'Fira Code'", 'monospace'],
      },
      spacing: {
        sidebar: '240px',
        header:  '64px',
        footer:  '40px',
      },
      borderRadius: {
        app: '8px',
      },
    },
  },
  plugins: [],
};
