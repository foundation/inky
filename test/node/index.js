/* eslint-env mocha */

const fs = require('fs');
const assert = require('assert');
const rimraf = require('rimraf');
const vfs = require('vinyl-fs');
const inky = require('../..');

describe('Inky wrappers', () => {
  const INPUT = 'test/fixtures/test.html';
  const OUTPUT = 'test/fixtures/_build';
  const OUTFILE = 'test/fixtures/_build/test.html';

  afterEach(done => {
    rimraf(OUTPUT, done);
  });

  it('can process a glob of files', () => {
    return inky({
      src: INPUT,
      dest: OUTPUT
    }).then(() => {
      assert(fs.existsSync(OUTFILE), 'Output file exists');
    });
  });

  it('can process a Gulp stream of files', done => {
    vfs.src(INPUT)
      .pipe(inky())
      .pipe(vfs.dest(OUTPUT))
      .on('finish', () => {
        assert(fs.existsSync(OUTFILE), 'Output file exists');
        done();
      });
  });
});
