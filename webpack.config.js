'use strict';

module.exports = {
  context: __dirname,
  entry: './lib/inky.js',
  output: {
    path: __dirname,
    filename: 'browser.js',
    library: {
      root: 'Inky',
      amd: 'inky',
      commonjs: 'inky'
    },
    libraryTarget: 'umd'
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        exclude: /node_modules/,
        use: {
          loader: 'babel-loader',
          options: {
            presets: [['env', {
              targets: {
                browsers: ['last 2 versions', 'ie >= 10']
              }
            }]]
          }
        }
      }
    ]
  }
};
