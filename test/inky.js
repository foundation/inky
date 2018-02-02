/* eslint-env mocha */

'use strict';

const assert = require('assert');
const fs = require('fs');
const rimraf = require('rimraf');
const vfs = require('vinyl-fs');
const Inky = require('../lib/inky');
const parse = require('..');
const compare = require('./lib/compare');

describe('Inky', () => {
  it('can take in settings in the constructor', () => {
    const config = {
      columnCount: 16
    };

    const inky = new Inky(config);

    assert.equal(inky.options.columnCount, 16, 'Sets a custom column count');
  });

  it(`doesn't choke on inline elements`, () => {
    const input = '<Container>This is a link to <a href="#">ZURB.com</a>.</Container>';
    const expected = `
      <table align="center" class="container">
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
    const input = '<Container>This is a link tö <a href="#">ZURB.com</a>.</Container>';
    const expected = `
      <table align="center" class="container">
        <tbody>
          <tr>
            <td>This is a link tö <a href="#">ZURB.com</a>.</td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it(`doesn't convert these characters into entities`, () => {
    const input = '<Container>There\'s &nbsp; some amazing things here!</Container>';
    const expected = `
      <table align="center" class="container">
        <tbody>
          <tr>
            <td>There's &nbsp; some amazing things here!</td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it(`doesn't decode entities if non default cheerio config is given`, () => {
    const input = '<Container>"should not replace quotes"</Container>';
    const expected = `
      <table align="center" class="container">
        <tbody>
          <tr>
            <td>"should not replace quotes"</td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected, {decodeEntities: false});
  });

  it(`doesn't muck with stuff inside raw`, () => {
    const input = '<raw><%= test %></raw>';
    const expected = '<%= test %>';

    compare(input, expected);
  });

  it(`can handle multiple raw tags`, () => {
    const input = '<h1><raw><%= test %></raw></h1><h2>< raw >!!!</ raw ></h2>';
    const expected = '<h1><%= test %></h1><h2>!!!</h2>';

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

  it('can process a glob of files', () => {
    return parse({
      src: INPUT,
      dest: OUTPUT
    }).then(() => {
      assert(fs.existsSync(OUTFILE), 'Output file exists');
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
