/* eslint-env browser */

'use strict';

const Inky = require('../lib/inky');

let inky;

window.setupInky = function (opts, cb) {
  opts = opts || {};
  opts.cheerio = Inky.mergeCheerioOpts(opts.cheerio);
  if (typeof inky === 'undefined') {
    inky = new Inky(opts);
  }

  // This transform function takes in an element and calls a callback.
  function transform(html, callback) {
    const convertedHtml = inky.releaseTheKraken(html, opts.cheerio);
    callback(null, convertedHtml);
  }

  cb(transform);
};

if (typeof (window) !== 'undefined') {
  window.runInky = function (opts, elem) {
    if (typeof (elem) === 'undefined') {
      elem = opts;
      opts = {};
    }
    window.setupInky(opts, transform => {
      transform(elem.outerHTML, (err, html) => {
        if (err === null) {
          elem.outerHTML = html;
        } else {
          console.log(err);
        }
      });
    });
  };
  const elems = document.body.getElementsByTagName('container');
  for (let i = 0; i < elems.length; i++) {
    window.runInky(elems[i]);
  }
}
