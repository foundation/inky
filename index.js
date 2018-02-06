'use strict';

const fs = require('fs');
const path = require('path');
const through = require('through2');
const vfs = require('vinyl-fs');
const mkdirp = require('mkdirp');
const requireDir = require('require-dir');
const cwd = require('prepend-cwd');
const Inky = require('./lib/inky');

let inky;

module.exports = function (opts) {
  const options = Object.assign({}, opts || {});

  if (typeof options.components === 'string') {
    const list = requireDir(cwd(options.components));
    options.components = Object.keys(list).map(k => list[k]);
  }

  if (typeof inky === 'undefined') {
    inky = new Inky(options);
  }

  // If the user passed in source files, create a stream
  if (options.src) {
    const stream = vfs
      .src(options.src)
      .pipe(transform());

    if (options.dest) {
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
      const convertedHtml = inky.releaseTheKraken(file.contents.toString());

      file.contents = Buffer.from(convertedHtml);

      // Write to disk manually if the user specified it
      if (options.dest) {
        const outputPath = path.join(options.dest, path.basename(file.path));
        mkdirp(options.dest, () => {
          fs.writeFile(outputPath, convertedHtml, cb);
        });
      } else {
        cb(null, file);
      }
    });
  }
};

module.exports.Inky = Inky;
