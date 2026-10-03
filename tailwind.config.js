/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // Free-CAT design tokens
        brand: {
          50:  '#f0f4ff',
          100: '#e0eaff',
          200: '#c7d7fe',
          300: '#a5bcfd',
          400: '#8097fa',
          500: '#6172f3', // primary
          600: '#4e56e8',
          700: '#3f44ce',
          800: '#3438a6',
          900: '#2f3483',
          950: '#1c1f4d',
        },
        // Editor segment status colours
        segment: {
          new:        '#f3f4f6',
          draft:      '#fef9c3',
          translated: '#dcfce7',
          review:     '#fee2e2',
          reviewed:   '#dbeafe',
          final:      '#f0fdf4',
          // Dark mode variants
          'new-dark':        '#1f2937',
          'draft-dark':      '#44400a',
          'translated-dark': '#14532d',
          'review-dark':     '#4c0519',
          'reviewed-dark':   '#1e3a5f',
          'final-dark':      '#052e16',
        },
        // TM match badge colours
        match: {
          exact:     '#16a34a', // 100%+
          high:      '#2563eb', // 85–99%
          medium:    '#d97706', // 70–84%
          low:       '#9ca3af', // <70%
        }
      },
      fontFamily: {
        editor: ['JetBrains Mono', 'Fira Code', 'Consolas', 'monospace'],
        ui:     ['Inter', 'system-ui', 'sans-serif'],
      },
      spacing: {
        'segment-row': '2.5rem',
      }
    }
  },
  plugins: []
};
