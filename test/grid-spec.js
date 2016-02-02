var Inky = require('../lib/inky');
var cheerio = require('cheerio');
var assert = require('assert');

describe("the container", function() {
  it("returns the correct container syntax", function() {
    var inky = new Inky();
    var $ = cheerio.load('<container></container>');

    $ = inky.releaseTheKraken($);
    compare($.html(), `
        <table class="container">
          <tbody>
            <tr>
              <td></td>
            </tr>
          </tbody>
        </table>`);
  });
});

describe("centering", function() {
  it("returns the correct centering syntax", function() {
    var inky = new Inky();
    var $ = cheerio.load('<center><table><tr><th></th></tr></table></center>');

    $ = inky.releaseTheKraken($);
    compare($.html(), `
        <center>
          <table class="center">
            <tbody>
              <tr>
                <th></th>
              </tr>
            </tbody>
          </table>
        </center>`);
  });
});


describe("the grid", function () {
  it("returns the correct row syntax", function() {
    var inky = new Inky();
    var $ = cheerio.load('<row></row>');

    $ = inky.releaseTheKraken($);
    compare($.html(), `
        <table class="row">
          <tbody>
            <tr></tr>
          </tbody>
        </table>`);
  });

  it("returns the correct sinlge column syntax", function () {
    var inky = new Inky();
    var $ = cheerio.load('<columns large="12" small="12"></columns>');

    $ = inky.releaseTheKraken($);
    compare($.html(), `
        <th class="columns small-12 large-12 first last">
          <table >
            <tr>
              <th class="expander"></th>
            </tr>
          </table>
        </th>`);
  });

  it("returns the correct two column syntax", function () {
    var inky = new Inky();
    var $ = cheerio.load('<columns large="6" small="12"></columns><columns large="6" small="12"></columns>');

    $ = inky.releaseTheKraken($);
    compare($.html(), `
        <th class="columns small-12 large-6 first">
          <table >
            <tr>
              <th class="expander"></th>
            </tr>
          </table>
        </th>
        <th class="columns small-12 large-6 last">
          <table >
            <tr>
              <th class="expander"></th>
            </tr>
          </table>
        </th>`);
  });
  
  it("returns the correct three column syntax", function () {
    var inky = new Inky();
    var $ = cheerio.load('<columns large="4" small="12"></columns><columns large="4" small="12"></columns><columns large="4" small="12"></columns>');

    $ = inky.releaseTheKraken($);
    compare($.html(), `
        <th class="columns small-12 large-4 first">
          <table >
            <tr>
              <th class="expander"></th>
            </tr>
          </table>
        </th>
        <th class="columns small-12 large-4">
          <table >
            <tr>
              <th class="expander"></th>
            </tr>
          </table>
        </th>
        <th class="columns small-12 large-4 last">
          <table >
            <tr>
              <th class="expander"></th>
            </tr>
          </table>
        </th>`);
  });

  it("returns the correct offset syntax", function () {
    var inky = new Inky();
    var $ = cheerio.load('');

    $ = inky.releaseTheKraken($);
    compare($.html(), `
        <th class="columns small-4 small-offset-8 large-4 large-offset-8">
        </th>`);
  });


  //if it just has small, borrow from small for large
  it("automatically assigns large columns if no large attribute is assigned", function() {
    var inky = new Inky();
    var $ = cheerio.load('<columns small="4"></columns><columns small="8"></columns>');

    $ = inky.releaseTheKraken($);
    compare($.html(), `
      <th class="columns small-4 large-4 first">
        <table >
          <tr>
            <th class="expander"></th>
          </tr>
        </table>
      </th>
      <th class="columns small-8 large-8 last">
        <table >
          <tr>
            <th class="expander"></th>
          </tr>
        </table>
      </th>`);
  });

  it("automatically assigns small columns as full width if only large defined", function() {
    var inky = new Inky();
    var $ = cheerio.load('<columns large="4"></columns><columns large="8"></columns>');

    var opts = {
      grid: 15
    }
    $ = inky.releaseTheKraken($, opts);
    compare($.html(), `
      <th class="columns small-12 large-4 first">
        <table >
          <tr>
            <th class="expander"></th>
          </tr>
        </table>
      </th>
      <th class="columns small-12 large-8 last">
        <table >
          <tr>
            <th class="expander"></th>
          </tr>
        </table>
      </th>`);
  });
});

describe("the block grid", function() {
  it("returns the correct block grid syntax", function() {
    var inky = new Inky();
    var $ = cheerio.load('<block-grid up="4"></block-grid>');

    $ = inky.releaseTheKraken($);
    compare($.html(), `
      <table class="block-grid up-4">
        <tr>
          <td></td>
          <td></td>
          <td></td>
          <td></td>
        </tr>
      </table>`);
  });
});

describe("buttons", function() {
  it("returns the correct generic button syntax", function() {
    var inky = new Inky();
    var $ = cheerio.load('<button href="http://zurb.com"></button>');

    $ = inky.releaseTheKraken($);
    compare($.html(), `
        <table class="button">
          <tr>
            <td>
              <table>
                <tr>
                  <td>
                    <a href="https://zurb.com">I am a button</a>
                  </td>
                </tr>
              </table>
            </td>
          </tr>
        </table>`);
  });

  //adds the classes for complex buttons
  it("returns the correct complex button syntax", function() {
    var inky = new Inky();
    var $ = cheerio.load('<button class="small alert expand" href="http://zurb.com"></button>');

    $ = inky.releaseTheKraken($);
    compare($.html(), `
        <table class="button small alert expand">
          <tr>
            <td>
              <table>
                <tr>
                  <td>
                    <a href="https://zurb.com">I am a button</a>
                  </td>
                </tr>
              </table>
            </td>
          </tr>
        </table>`);
  });
});

describe("menus", function() {
  it("returns the correct complete menu syntax", function() {
    var inky = new Inky();
    var $ = cheerio.load('<menu><item href="http://zurb.com">My Item 1</item><item href="http://zurb.com">My Item 2</item><item href="http://zurb.com">My Item 3</item></menu>');

    $ = inky.releaseTheKraken($);
    compare($.html(), `
      <table class="menu">
        <tr>
          <td>
            <a href="http://zurb.com">My Item 1</a>
          </td>
          <td>
            <a href="http://zurb.com">My Item 2</a>
          </td>
          <td>
            <a href="http://zurb.com">My Item 3</a>
          </td>
        </tr>
      </table>`);
  });

  it("returns the correct menu wrapper with custom syntax", function() {
    var inky = new Inky();
    var $ = cheerio.load('<menu><item href="http://zurb.com">My Item 1</item><td><a href="http://zurb.com">My Item 2</a></td><item href="http://zurb.com">My Item 3</item></menu>');

    $ = inky.releaseTheKraken($);
    compare($.html(), `
      <table class="menu">
        <tr>
          <td>
            <a href="http://zurb.com">My Item 1</a>
          </td>
          <td>
            <a href="http://zurb.com">My Item 2</a>
          </td>
          <td>
            <a href="http://zurb.com">My Item 3</a>
          </td>
        </tr>
      </table>`);
  });
});

function compare(expected, actual) {
  assert.equal(expected, oneLine(actual));
}

// Thank you: https://muffinresearch.co.uk/removing-leading-whitespace-in-es6-template-strings/
function oneLine(string) {
  var output = '';

  // Split on newlines.
  var lines = string.split(/(?:\r\n|\n|\r)/);

  // Rip out the leading whitespace.
  return lines.map((line) => {
    return line.replace(/^\s+/gm, '');
  }).join('').trim();
}
