var Inky = require('../lib/inky');
var cheerio = require('cheerio');
var assert = require('assert');

describe('Inky', function () {
  it('can take in settings in the constructor', function() {
    var config = {
      components: { column: 'col' },
      attributes: ['href', 'disabled'],
      columnCount: 16
    }

    var inky = new Inky(config);

    assert.equal(inky.components.column, 'col', 'Sets custom component tags');
    assert.deepEqual(inky.attributes, ['href', 'disabled'], 'Sets custom attributes to ignore');
    assert.equal(inky.columnCount, 16, 'Sets a custom column count');
  });

  it('should be setting custom tags from object correctly', function() {
    var inky = new Inky();
    inky.setTagArray();
    assert.deepEqual(inky.zfArray, ['button', 'row', 'callout', 'columns', 'subcolumns', 'container', 'inky', 'block-grid', 'menu', 'item']);
  });
});
