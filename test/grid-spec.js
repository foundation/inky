var Inky = require('../lib/inky');
var cheerio = require('cheerio');
var assert = require('assert');
var htmlEqual = require('assert-html-equal');

describe('Container', function() {
  it('creates a container table', function() {
    var input = '<container></container>';
    var expected = `
      <table class="container">
        <tbody>
          <tr>
            <td></td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });
});

describe('Centering', function() {
  it('creates a properly centered table', function() {
    var input = `
      <center>
        <table>
          <tr>
            <th></th>
          </tr>
        </table>
      </center>
    `;
    var expected = `
      <center>
        <table class="center">
          <tbody>
            <tr>
              <th></th>
            </tr>
          </tbody>
        </table>
      </center>
    `;

    compare(input, expected);
  });
});


describe('Grid', function() {
  it('creates a row', function() {
    var input = '<row></row>';
    var expected =  `
      <table class="row">
        <tbody>
          <tr></tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it('creates a single column with first and last classes', function () {
    var input = '<columns large="12" small="12"></columns>';
    var expected = `
      <th class="small-12 large-12 columns first last">
        <table >
          <tr>
            <th class="expander"></th>
          </tr>
        </table>
      </th>
    `;

    compare(input, expected);
  });

  it('creates two columns, one first, one last', function () {
    var input = `
      <columns large="6" small="12"></columns>
      <columns large="6" small="12"></columns>
    `;
    var expected = `
      <th class="small-12 large-6 columns first">
        <table >
          <tr>
            <th class="expander"></th>
          </tr>
        </table>
      </th>
      <th class="small-12 large-6 columns last">
        <table >
          <tr>
            <th class="expander"></th>
          </tr>
        </table>
      </th>
    `;

    compare(input, expected);
  });

  it('creates 3+ columns, first is first, last is last', function() {
    var input = `
      <columns large="4" small="12"></columns>
      <columns large="4" small="12"></columns>
      <columns large="4" small="12"></columns>
    `;
    var expected = `
      <th class="small-12 large-4 columns first">
        <table >
          <tr>
            <th class="expander"></th>
          </tr>
        </table>
      </th>
      <th class="small-12 large-4 columns">
        <table >
          <tr>
            <th class="expander"></th>
          </tr>
        </table>
      </th>
      <th class="small-12 large-4 columns last">
        <table >
          <tr>
            <th class="expander"></th>
          </tr>
        </table>
      </th>
    `;

    compare(input, expected);
  });

  it('offsets a column', function() {
    var input = '';
    var expected = `
      <th class="small-4 small-offset-8 large-4 large-offset-8 columns">
      </th>
    `;

    compare(input, expected)
  });


  //if it just has small, borrow from small for large
  it('automatically assigns large columns if no large attribute is assigned', function() {
    var input = `
      <columns small="4"></columns>
      <columns small="8"></columns>
    `;
    var expected = `
      <th class="small-4 large-4 columns first">
        <table >
          <tr>
            <th class="expander"></th>
          </tr>
        </table>
      </th>
      <th class="small-8 large-8 columns last">
        <table >
          <tr>
            <th class="expander"></th>
          </tr>
        </table>
      </th>
    `;

    compare(input, expected);
  });

  it('automatically assigns small columns as full width if only large defined', function() {
    var input = `
      <columns large="4"></columns>
      <columns large="8"></columns>
    `;
    var expected = `
      <th class="small-12 large-4 columns first">
        <table >
          <tr>
            <th class="expander"></th>
          </tr>
        </table>
      </th>
      <th class="small-12 large-8 columns last">
        <table >
          <tr>
            <th class="expander"></th>
          </tr>
        </table>
      </th>
    `;

    compare(input, expected);

    // ?
    // var opts = {
    //   grid: 15
    // }
    // $ = inky.releaseTheKraken($, opts);
  });
});

describe('Block Grid', function() {
  it('returns the correct block grid syntax', function() {
    var input = '<block-grid up="4"></block-grid>';
    var expected = `
      <table class="block-grid up-4">
        <tr>
          <td></td>
          <td></td>
          <td></td>
          <td></td>
        </tr>
      </table>
    `;

    compare(input, expected);
  });
});

describe('Button', function() {
  it('creates a simple button', function() {
    var input = '<button href="http://zurb.com">Button</button>';
    var expected = `
      <table class="button">
        <tr>
          <td>
            <table>
              <tr>
                <td><a href="http://zurb.com">Button</a></td>
              </tr>
            </table>
          </td>
        </tr>
      </table>
    `;

    compare(input, expected);
  });

  it('creates a button with classes', function() {
    var input = `
      <button class="small alert expand" href="http://zurb.com">Button</button>
    `;
    var expected = `
      <table class="button small alert expand">
        <tr>
          <td>
            <table>
              <tr>
                <td><a href="http://zurb.com">Button</a></td>
              </tr>
            </table>
          </td>
        </tr>
      </table>
    `;

    compare(input, expected);
  });
});

describe('Menu', function() {
  it('creates a menu with item tags inside', function() {
    var input = `
      <menu>
        <item href="http://zurb.com">Item</item>
      </menu>
    `;
    var expected = `
      <table class="menu">
        <tr>
          <td>
            <a href="http://zurb.com">Item</a>
          </td>
        </tr>
      </table>
    `;

    compare(input, expected);
  });

  it('works without using an item tag', function() {
    var input = `
      <menu>
        <item href="http://zurb.com">Item 1</item>
        <td><a href="http://zurb.com">Item 2</a></td>
      </menu>
    `;
    var expected = `
      <table class="menu">
        <tr>
          <td>
            <a href="http://zurb.com">Item 1</a>
          </td>
          <td>
            <a href="http://zurb.com">Item 2</a>
          </td>
        </tr>
      </table>
    `;

    compare(input, expected);
  });
});

function compare(input, expected) {
  var inky = new Inky();
  var $ = cheerio.load(input);
  var output = inky.releaseTheKraken($).html();

  htmlEqual(output, expected);
}
