/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          purple: '#8B5CF6',
          pink: '#EC4899',
        },
        accent: {
          yellow: '#FBBF24',
          mint: '#34D399',
        },
        bg: {
          dark: '#0F172A',
          surface: '#1E293B',
        },
      },
      fontFamily: {
        display: ['system-ui', 'sans-serif'],
      },
      animation: {
        'pulse-slow': 'pulse 3s infinite',
        'bounce-slow': 'bounce 2s infinite',
      },
      backgroundImage: {
        'gradient-main': 'linear-gradient(135deg, #8B5CF6 0%, #EC4899 50%, #F97316 100%)',
      },
    },
  },
  plugins: [],
}
