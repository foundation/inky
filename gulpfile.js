var browserify = require('browserify');
var gulp = require('gulp');
var source = require('vinyl-source-stream');
var buffer = require('vinyl-buffer');
var uglify = require('gulp-uglify');

gulp.task('browser', function() {
  var b = browserify({
    entries: 'bin/inky-browser.js',
    debug: false
  });

  return b.bundle()
    .pipe(source('inky-browser.js'))
    .pipe(buffer())
    .pipe(uglify())
    .pipe(gulp.dest('./dist/'));
});

