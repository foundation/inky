var cheerio = require('cheerio');
var Inky = require('../lib/inky');

var inky;

window.setupInky = function(opts, cb) {
  opts = opts || {};
  opts.cheerio = Inky.mergeCheerioOpts(opts.cheerio);
  if (typeof inky === 'undefined') {
    inky = new Inky(opts);
  }

// This transform function takes in an element and calls a callback.
  function transform(html, callback) {
    var convertedHtml = inky.releaseTheKraken(html, opts.cheerio);
    callback(null, convertedHtml);
  }

  cb(transform);
}

if(typeof(window) !== 'undefined') {
  window.runInky = function(opts, elem) {
    if(typeof(elem) === 'undefined') {
      elem = opts;
      opts = {};
    }
    window.setupInky(opts, function(transform) {
      transform(elem.outerHTML, function(err, html) {
        if(err === null) {
          elem.outerHTML = html;
        } else {
          console.log(err);
        }
      });
    });
  }
  var elems = document.body.getElementsByTagName('container')
  for(var i = 0; i < elems.length; i++) {
    window.runInky(elems[i]);
  }
}

