/* eslint-env mocha */
/* eslint-disable no-unused-expressions */

const fs = require('fs');
const path = require('path');
const assert = require('assert');
const expect = require('chai').expect;
const vfs = require('vinyl-fs');
const tempy = require('tempy');

let inky;

describe('Inky Node', () => {
  const testFile = 'test.html';
  const input = path.join('test/fixtures/basic', testFile);

  beforeEach(() => {
    // Because this module is a singleton, we have to reload it for each test to get a clean slate
    inky = require('../..');
  });

  afterEach(() => {
    delete require.cache[require.resolve('../..')];
  });

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

  it('can load a folder of custom components', () => {
    const dir = tempy.directory();

    return inky({
      src: path.join('test/fixtures/custom-components', testFile),
      components: 'test/fixtures/components',
      dest: dir
    }).then(() => {
      const contents = fs.readFileSync(path.join(dir, testFile)).toString();
      expect(contents).to.contain('<div class="mock">');
    });
  });
});
