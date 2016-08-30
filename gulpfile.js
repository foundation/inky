var browserify = require('browserify');
var gulp = require('gulp');
var source = require('vinyl-source-stream');
var buffer = require('vinyl-buffer');

gulp.task('browser', function() {
  var b = browserify({
    entries: 'bin/inky-browser.js',
    debug: true
  });

  return b.bundle()
    .pipe(source('inky-browser.js'))
    .pipe(buffer())
    .pipe(gulp.dest('./dist/'));
});

