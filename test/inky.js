var Inky = require('../lib/inky');
var parse = require('..');
var cheerio = require('cheerio');
var assert = require('assert');
var fs = require('fs');
var rimraf = require('rimraf');
var vfs = require('vinyl-fs');
var exec = require('child_process').exec;

describe('Inky', function() {
  it('can take in settings in the constructor', function() {
    var config = {
      components: { column: 'col' },
      attributes: ['href', 'disabled'],
      columnCount: 16
    }

    var inky = new Inky(config);

    assert.equal(inky.components.column, 'col', 'Sets custom component tags');
    assert.equal(inky.columnCount, 16, 'Sets a custom column count');
  });

  it('should be setting custom tags from object correctly', function() {
    var inky = new Inky();
    inky.setTagArray();
    assert.deepEqual(inky.zfArray, ['button', 'row', 'callout', 'columns', 'subcolumns', 'container', 'inky', 'block-grid', 'menu', 'item']);
  });
});

describe('Inky wrappers', function() {
  afterEach(function(done) {
    rimraf('test/fixtures/_build', done);
  });

  it('can process a glob of files', function(done) {
    parse({
      src: 'test/fixtures/test.html',
      dest: 'test/fixtures/_build'
    }, function() {
      assert(fs.existsSync('test/fixtures/_build/test.html'), 'Output file exists');
      done();
    });
  });

  it('can process a Gulp stream of files', function(done) {
    vfs.src('test/fixtures/test.html')
      .pipe(parse())
      .pipe(vfs.dest('test/fixtures/_build'))
      .on('finish', function() {
        assert(fs.existsSync('test/fixtures/_build/test.html'), 'Output file exists');
        done();
      });
  });

  it.only('works as a CLI', function(done) {
    exec('bin/inky.js test/fixtures/test.html test/fixtures/_build --watch', function(e, o, r) {
      console.log(o);
      done();
    });
  });
});
