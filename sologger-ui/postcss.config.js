export default {
  plugins: {
    'tailwindcss': {},
    'postcss-import': {},
    'autoprefixer': {},
    'postcss-preset-env': {
      features: {
        'nesting-rules': true,
      },
    },
    'postcss-custom-media': {

          customMedia: {
            '--sm': '(min-width: 640px)',
            '--md': '(min-width: 768px)',
            '--lg': '(min-width: 1024px)',
            '--xl': '(min-width: 1280px)',
            '--2xl': '(min-width: 1536px)',
          },
    },
  },
}