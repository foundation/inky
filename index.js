var cheerio = require('cheerio');
var path = require('path');
var through = require('through2');
var vfs = require('vinyl-fs');
var Inky = require('./lib/inky');

var inky = new Inky();

module.exports = function(opts, cb) {
  var stream;
  opts = opts || {};

  // If the user passed in source files, create a stream
  if (opts.src) {
    stream = vfs
      .src(opts.src)
      .pipe(process());

    if (opts.dest && typeof cb === 'function') {
      stream.on('finish', cb);
    }
  }
  // Otherwise, return the transform function
  else {
    return transform();
  }

  // This transform function takes in a Vinyl HTML file, converts the code from Inky to HTML, and returns the modified file.
  function transform() {
    return through.obj(function(file, enc, callback) {
      var html = cheerio.load(file.contents.toString());
      var convertedHtml = inky.releaseTheKraken(html, opts);

      file.contents = new Buffer(convertedHtml.html());

      // Write to disk manually if the user specified it
      if (opts.dest) {
        var outputPath = path.join(opts.dest, path.basename(file.path));
        fs.writeFile(outPath, convertedHtml, callback);
      }
      else {
        callback(null, file);
      }
    });
  }
}

module.exports.Inky = Inky;
