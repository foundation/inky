var Inky = require('../lib/inky');
var parse = require('..');
var cheerio = require('cheerio');
var assert = require('assert');
var fs = require('fs');
var rimraf = require('rimraf');
var vfs = require('vinyl-fs');
var exec = require('child_process').exec;
var compare = require('./lib/compare');

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

  it('should have an array of component tags', () => {
    var inky = new Inky();
    assert(Array.isArray(inky.componentTags), 'Inky.zftags is an array');
  });

  it(`doesn't choke on inline elements`, () => {
    var input = '<container>This is a link to <a href="#">ZURB.com</a>.</container>';
    var expected = `
      <table class="container">
        <tbody>
          <tr>
            <td>This is a link to <a href="#">ZURB.com</a>.</td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it(`doesn't choke on special characters`, () => {
    var input = '<container>This is a link tö <a href="#">ZURB.com</a>.</container>';
    var expected = `
      <table class="container">
        <tbody>
          <tr>
            <td>This is a link t&#xF6; <a href="#">ZURB.com</a>.</td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
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
});
