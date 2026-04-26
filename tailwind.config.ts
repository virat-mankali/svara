import type { Config } from 'tailwindcss';

export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        ink: '#17211b',
        moss: '#516a58',
        leaf: '#2f7d56',
        paper: '#fbfaf6',
        ember: '#d66a3a',
      },
      boxShadow: {
        panel: '0 18px 45px rgba(23, 33, 27, 0.12)',
      },
    },
  },
  plugins: [],
} satisfies Config;
