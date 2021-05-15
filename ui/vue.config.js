module.exports = {
  publicPath: '/wikit',
  chainWebpack: config => {
      config.module.rules.delete('eslint');
  },
  configureWebpack: config => {
    config.entry.app = ['babel-polyfill', './src/main.js'];
  },
  pages: {
    index: {
      entry: 'src/main.js',
      title: 'Wikit (Beta)',
    }
  },
}
