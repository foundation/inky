var cheerio = require('cheerio');
var Inky = require('../lib/inky');

var inky;

var setupInky = function(opts, cb) {
  opts = opts || {};
  opts.cheerio = Inky.mergeCheerioOpts(opts.cheerio);
  if (typeof inky === 'undefined') {
    inky = new Inky(opts);
  }

// This transform function takes in an element and calls a callback.
  function transform(html, callback) {
    inky.releaseTheKraken(html, opts.cheerio, function(convertedHtml) {
      callback(null, convertedHtml);
    });
  }

  return transform;
}

if(typeof(window) !== 'undefined') {
  window.runInky = function(opts, elem) {
    if(typeof(elem) === 'undefined') {
      elem = opts;
      opts = {};
    }
    setupInky(opts, function(transform) {
      transform(elem.outerHTML, function(err, html) {
        if(error === null) {
          elem.outerHTML = html;
        }
      });
    });
  }
}

