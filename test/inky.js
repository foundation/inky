var Inky = require('../lib/inky');
var cheerio = require('cheerio');
var assert = require('assert');

describe("inky", function () {
  it("should be setting custom tags from object correctly", function () {
    var inky = new Inky();
    inky.setTagArray();
    assert.deepEqual(inky.zfArray, ['button', 'row', 'callout', 'columns', 'subcolumns', 'container', 'inline-list-h', 'inline-list-v', 'inky']);
  });
});
