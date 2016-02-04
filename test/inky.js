var Inky = require('../lib/inky');
var cheerio = require('cheerio');
var assert = require('assert');

describe('Inky', function () {
  it('Should be setting custom tags from object correctly', function() {
    var inky = new Inky();
    inky.setTagArray();
    assert.deepEqual(inky.zfArray, ['button', 'row', 'callout', 'columns', 'subcolumns', 'container', 'inky', 'block-grid', 'menu', 'item']);
  });
});
