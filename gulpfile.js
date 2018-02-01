const browserify = require('browserify');
const gulp = require('gulp');
const source = require('vinyl-source-stream');
const buffer = require('vinyl-buffer');
const uglify = require('gulp-uglify');

gulp.task('browser', () => {
  const b = browserify({
    entries: 'bin/inky-browser.js',
    debug: false
  });

  return b.bundle()
    .pipe(source('inky-browser.js'))
    .pipe(buffer())
    .pipe(uglify())
    .pipe(gulp.dest('./dist/'));
});

