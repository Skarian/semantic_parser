/** @type {import('tailwindcss').Config} */

import catppuccin from '@catppuccin/daisyui'

export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {},
  },
  plugins: [
    require('daisyui')
  ],
  daisyui: {
    themes: [
      catppuccin('mocha', { primary: 'sky', secondary: 'rosewater' })
    ]
  }
}

