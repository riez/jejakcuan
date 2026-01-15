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
    '--theme-font-color-base': '0 0 0',
    '--theme-font-color-dark': '255 255 255',
    '--theme-rounded-base': '8px',
    '--theme-rounded-container': '12px',
    '--theme-border-base': '1px',
    // =~= Theme On-X Colors =~=
    '--on-primary': '255 255 255',
    '--on-secondary': '255 255 255',
    '--on-tertiary': '0 0 0',
    '--on-success': '0 0 0',
    '--on-warning': '0 0 0',
    '--on-error': '255 255 255',
    '--on-surface': '255 255 255',
    // =~= Theme Colors =~=
    // Primary: Deep Blue (trust, finance)
    '--color-primary-50': '226 232 240',
    '--color-primary-100': '216 224 235',
    '--color-primary-200': '206 217 230',
    '--color-primary-300': '177 194 215',
    '--color-primary-400': '118 148 185',
    '--color-primary-500': '59 102 155', // Main primary
    '--color-primary-600': '53 92 140',
    '--color-primary-700': '44 77 116',
    '--color-primary-800': '36 61 93',
    '--color-primary-900': '29 50 76',
    // Secondary: Teal (growth, positive)
    '--color-secondary-50': '225 245 243',
    '--color-secondary-100': '215 241 239',
    '--color-secondary-200': '205 238 235',
    '--color-secondary-300': '175 227 223',
    '--color-secondary-400': '114 206 198',
    '--color-secondary-500': '54 185 174', // Main secondary
    '--color-secondary-600': '49 167 157',
    '--color-secondary-700': '41 139 130',
    '--color-secondary-800': '32 111 104',
    '--color-secondary-900': '26 91 85',
    // Tertiary: Gold (premium, value)
    '--color-tertiary-50': '252 247 230',
    '--color-tertiary-100': '251 244 222',
    '--color-tertiary-200': '250 241 213',
    '--color-tertiary-300': '247 233 188',
    '--color-tertiary-400': '240 217 138',
    '--color-tertiary-500': '234 200 88', // Main tertiary (gold)
    '--color-tertiary-600': '211 180 79',
    '--color-tertiary-700': '176 150 66',
    '--color-tertiary-800': '140 120 53',
    '--color-tertiary-900': '115 98 43',
    // Success: Green (profit, buy)
    '--color-success-50': '227 245 233',
    '--color-success-100': '218 242 225',
    '--color-success-200': '208 239 218',
    '--color-success-300': '180 229 195',
    '--color-success-400': '124 209 151',
    '--color-success-500': '68 189 106', // Main success
    '--color-success-600': '61 170 95',
    '--color-success-700': '51 142 80',
    '--color-success-800': '41 113 64',
    '--color-success-900': '33 93 52',
    // Warning: Amber (caution, hold)
    '--color-warning-50': '252 245 231',
    '--color-warning-100': '251 241 222',
    '--color-warning-200': '250 238 214',
    '--color-warning-300': '247 228 189',
    '--color-warning-400': '240 207 140',
    '--color-warning-500': '234 186 90', // Main warning
    '--color-warning-600': '211 167 81',
    '--color-warning-700': '176 140 68',
    '--color-warning-800': '140 112 54',
    '--color-warning-900': '115 91 44',
    // Error: Red (loss, sell)
    '--color-error-50': '249 229 229',
    '--color-error-100': '247 220 220',
    '--color-error-200': '245 212 212',
    '--color-error-300': '238 186 186',
    '--color-error-400': '226 133 133',
    '--color-error-500': '213 81 81', // Main error
    '--color-error-600': '192 73 73',
    '--color-error-700': '160 61 61',
    '--color-error-800': '128 49 49',
    '--color-error-900': '104 40 40',
    // Surface: Dark slate (professional, dashboard)
    '--color-surface-50': '243 244 246',
    '--color-surface-100': '229 231 235',
    '--color-surface-200': '209 213 219',
    '--color-surface-300': '156 163 175',
    '--color-surface-400': '107 114 128',
    '--color-surface-500': '75 85 99',
    '--color-surface-600': '55 65 81',
    '--color-surface-700': '45 55 72',
    '--color-surface-800': '30 41 59',
    '--color-surface-900': '15 23 42',
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
