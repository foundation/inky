/* eslint-env mocha */

const assert = require('assert');
const $ = require('cheerio');
const renderer = require('../../lib/renderer');

const opts = {};
const components = {
  mock: {
    name: 'Mock',
    props: {
      class: ''
    },
    render(element, props) {
      return `<div class="mock ${props.class}">${props.children()}</div>`;
    }
  }
};
const library = new Map(
  Object.keys(components).map(k => [components[k].name, components[k]])
);

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
