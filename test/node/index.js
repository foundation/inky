/* eslint-env mocha */

const fs = require('fs');
const path = require('path');
const assert = require('assert');
const vfs = require('vinyl-fs');
const tempy = require('tempy');
const inky = require('../..');

describe('Inky Node', () => {
  const testFile = 'test.html';
  const input = path.join('test/fixtures', testFile);

  it('can process a glob of files', () => {
    const dir = tempy.directory();

    return inky({
      src: input,
      dest: dir
    }).then(() => {
      assert(fs.existsSync(path.join(dir, testFile)), 'Output file exists');
    });
  });

  it('can process a Gulp stream of files', done => {
    const dir = tempy.directory();

    vfs.src(input)
      .pipe(inky())
      .pipe(vfs.dest(dir))
      .on('finish', () => {
        assert(fs.existsSync(path.join(dir, testFile)), 'Output file exists');
        done();
      });
  });
});
