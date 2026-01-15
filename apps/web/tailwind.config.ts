import { join } from 'path';
import type { Config } from 'tailwindcss';
import { skeleton } from '@skeletonlabs/tw-plugin';
import forms from '@tailwindcss/forms';
import type { CustomThemeConfig } from '@skeletonlabs/tw-plugin';

// JejakCuan Custom Theme - Financial/Stock Trading aesthetic
const jejakCuanTheme: CustomThemeConfig = {
  name: 'jejakcuan',
  properties: {
    // =~= Theme Properties =~=
    '--theme-font-family-base': 'Inter, ui-sans-serif, system-ui, sans-serif',
    '--theme-font-family-heading': 'Inter, ui-sans-serif, system-ui, sans-serif',
    '--theme-font-color-base': '15 23 42', // Darker for better contrast on light bg
    '--theme-font-color-dark': '248 250 252', // Brighter white for dark mode
    '--theme-rounded-base': '8px',
    '--theme-rounded-container': '12px',
    '--theme-border-base': '1px',
    // =~= Theme On-X Colors =~=
    '--on-primary': '255 255 255',
    '--on-secondary': '255 255 255',
    '--on-tertiary': '15 23 42',
    '--on-success': '15 23 42', // Dark text on success for better contrast
    '--on-warning': '15 23 42',
    '--on-error': '255 255 255',
    '--on-surface': '248 250 252', // Bright text on dark surfaces
    // =~= Theme Colors =~=
    // Primary: Deep Blue (trust, finance) - WCAG AA compliant
    '--color-primary-50': '239 246 255',
    '--color-primary-100': '219 234 254',
    '--color-primary-200': '191 219 254',
    '--color-primary-300': '147 197 253',
    '--color-primary-400': '96 165 250', // Brighter for links (WCAG AA on dark)
    '--color-primary-500': '59 130 246', // Main primary - good contrast
    '--color-primary-600': '37 99 235',
    '--color-primary-700': '29 78 216',
    '--color-primary-800': '30 64 175',
    '--color-primary-900': '30 58 138',
    // Secondary: Teal (growth, positive)
    '--color-secondary-50': '240 253 250',
    '--color-secondary-100': '204 251 241',
    '--color-secondary-200': '153 246 228',
    '--color-secondary-300': '94 234 212',
    '--color-secondary-400': '45 212 191',
    '--color-secondary-500': '20 184 166', // Main secondary
    '--color-secondary-600': '13 148 136',
    '--color-secondary-700': '15 118 110',
    '--color-secondary-800': '17 94 89',
    '--color-secondary-900': '19 78 74',
    // Tertiary: Gold/Amber (premium, value) - Improved contrast
    '--color-tertiary-50': '255 251 235',
    '--color-tertiary-100': '254 243 199',
    '--color-tertiary-200': '253 230 138',
    '--color-tertiary-300': '252 211 77',
    '--color-tertiary-400': '251 191 36',
    '--color-tertiary-500': '245 158 11', // Main tertiary (amber)
    '--color-tertiary-600': '217 119 6',
    '--color-tertiary-700': '180 83 9',
    '--color-tertiary-800': '146 64 14',
    '--color-tertiary-900': '120 53 15',
    // Success: Green (profit, buy) - WCAG AA compliant, brighter for dark mode
    '--color-success-50': '240 253 244',
    '--color-success-100': '220 252 231',
    '--color-success-200': '187 247 208',
    '--color-success-300': '134 239 172',
    '--color-success-400': '74 222 128', // Brighter green for dark backgrounds
    '--color-success-500': '34 197 94', // Main success - good visibility
    '--color-success-600': '22 163 74',
    '--color-success-700': '21 128 61',
    '--color-success-800': '22 101 52',
    '--color-success-900': '20 83 45',
    // Warning: Amber (caution, hold) - Better contrast
    '--color-warning-50': '255 251 235',
    '--color-warning-100': '254 243 199',
    '--color-warning-200': '253 230 138',
    '--color-warning-300': '252 211 77',
    '--color-warning-400': '251 191 36',
    '--color-warning-500': '245 158 11', // Main warning
    '--color-warning-600': '217 119 6',
    '--color-warning-700': '180 83 9',
    '--color-warning-800': '146 64 14',
    '--color-warning-900': '120 53 15',
    // Error: Red (loss, sell) - WCAG AA compliant, brighter for visibility
    '--color-error-50': '254 242 242',
    '--color-error-100': '254 226 226',
    '--color-error-200': '254 202 202',
    '--color-error-300': '252 165 165',
    '--color-error-400': '248 113 113', // Brighter red for dark backgrounds
    '--color-error-500': '239 68 68', // Main error - high visibility
    '--color-error-600': '220 38 38',
    '--color-error-700': '185 28 28',
    '--color-error-800': '153 27 27',
    '--color-error-900': '127 29 29',
    // Surface: Dark slate (professional, dashboard) - Better text contrast
    '--color-surface-50': '248 250 252', // Brightest
    '--color-surface-100': '241 245 249',
    '--color-surface-200': '226 232 240',
    '--color-surface-300': '203 213 225', // Good for secondary text in dark mode
    '--color-surface-400': '148 163 184', // Muted text - still readable
    '--color-surface-500': '100 116 139',
    '--color-surface-600': '71 85 105',
    '--color-surface-700': '51 65 85',
    '--color-surface-800': '30 41 59',
    '--color-surface-900': '15 23 42', // Darkest
  }
};

const config = {
  darkMode: 'class',
  content: [
    './src/**/*.{html,js,svelte,ts}',
    join(require.resolve('@skeletonlabs/skeleton'), '../**/*.{html,js,svelte,ts}')
  ],
  theme: {
    extend: {
      fontFamily: {
        sans: ['Inter', 'ui-sans-serif', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'ui-monospace', 'monospace'],
      },
    }
  },
  plugins: [
    forms,
    skeleton({
      themes: {
        custom: [jejakCuanTheme],
        preset: ['skeleton']
      }
    })
  ]
} satisfies Config;

export default config;
