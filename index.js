'use strict';

const fs = require('fs');
const path = require('path');
const through = require('through2');
const vfs = require('vinyl-fs');
const mkdirp = require('mkdirp');
const Inky = require('./lib/inky');

let inky;

module.exports = function (opts) {
  opts = opts || {};

  if (typeof inky === 'undefined') {
    inky = new Inky(opts);
  }

  // If the user passed in source files, create a stream
  if (opts.src) {
    const stream = vfs
      .src(opts.src)
      .pipe(transform());

    if (opts.dest) {
      return new Promise((resolve, reject) => {
        stream.on('finish', resolve);
        stream.on('error', reject);
      });
    }
  } else {
		// Otherwise, return the transform function
    return transform();
  }

  // This transform function takes in a Vinyl HTML file, converts the code from Inky to HTML, and returns the modified file.
  function transform() {
    return through.obj((file, enc, cb) => {
      const convertedHtml = inky.releaseTheKraken(file.contents.toString(), opts.cheerio);

      file.contents = Buffer.from(convertedHtml);

      // Write to disk manually if the user specified it
      if (opts.dest) {
        const outputPath = path.join(opts.dest, path.basename(file.path));
        mkdirp(opts.dest, () => {
          fs.writeFile(outputPath, convertedHtml, cb);
        });
      } else {
        cb(null, file);
      }
    });
  }
};

module.exports.Inky = Inky;
