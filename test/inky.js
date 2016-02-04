var Inky = require('../lib/inky');
var parse = require('..');
var cheerio = require('cheerio');
var assert = require('assert');
var fs = require('fs');
var rimraf = require('rimraf');
var vfs = require('vinyl-fs');
var exec = require('child_process').exec;

describe('Inky', () => {
  it('can take in settings in the constructor', () => {
    var config = {
      components: { column: 'col' },
      columnCount: 16
    }

    var inky = new Inky(config);

    assert.equal(inky.components.column, 'col', 'Sets custom component tags');
    assert.equal(inky.columnCount, 16, 'Sets a custom column count');
  });

  it('should be setting custom tags from object correctly', () => {
    var inky = new Inky();
    inky.setTagArray();
    assert.deepEqual(inky.zfArray, ['button', 'row', 'callout', 'columns', 'subcolumns', 'container', 'inky', 'block-grid', 'menu', 'item']);
  });
});

describe('Inky wrappers', () => {
  const INPUT = 'test/fixtures/test.html';
  const OUTPUT = 'test/fixtures/_build';
  const OUTFILE = 'test/fixtures/_build/test.html';

  afterEach(done => {
    rimraf(OUTPUT, done);
  });

  it('can process a glob of files', done => {
    parse({
      src: INPUT,
      dest: OUTPUT
    }, () => {
      assert(fs.existsSync(OUTFILE), 'Output file exists');
      done();
    });
  });

  it('can process a Gulp stream of files', done => {
    vfs.src(INPUT)
      .pipe(parse())
      .pipe(vfs.dest(OUTPUT))
      .on('finish', () => {
        assert(fs.existsSync(OUTFILE), 'Output file exists');
        done();
      });
  });

  it('works as a CLI', done => {
    exec(`bin/inky.js ${INPUT} ${OUTPUT}`, () => {
      assert(fs.existsSync(OUTFILE), 'Output file exists');
      done();
    });
  });
});
