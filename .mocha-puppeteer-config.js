module.exports = {
  lassoConfig: {
    require: {
      transforms: [
        'lasso-babel-transform',
      ],
    },
  },
  lassoDependencies: [
    './browser.js'
  ]
};
