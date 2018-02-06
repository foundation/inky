/* eslint-env mocha */

'use strict';

const expect = require('chai').expect;
const Inky = require('../../lib/inky');
const compare = require('../lib/compare');

describe('Inky', () => {
  it('can take in settings in the constructor', () => {
    const config = {
      columnCount: 16
    };
    const inky = new Inky(config);

    expect(inky.options.columnCount).to.equal(16);
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

  it('allows custom components to be added', () => {
    const components = [{
      name: 'Mock',
      render: () => '<div class="mock"></div>'
    }];
    const input = '<Mock></Mock>';
    const expected = components[0].render();

    compare(input, expected, {components});
  });
});
