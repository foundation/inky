/* eslint-env mocha */

const assert = require('assert');
const $ = require('cheerio');
const renderer = require('../../lib/renderer');

const opts = {};
const library = new Map([['Mock', {
  name: 'Mock',
  props: {
    class: ''
  },
  render(props) {
    return `<div class="mock ${props.class}">${props.children()}</div>`;
  }
}]]);

describe('renderer', () => {
  it('works', () => {
    const render = renderer(library, opts);
    const actual = render($.load('<Mock class="custom">Hi</Mock>', {
      lowerCaseTags: false
    })('Mock'));
    const expected = '<div class="mock custom">Hi</div>';

    assert.equal(actual, expected);
  });
});
