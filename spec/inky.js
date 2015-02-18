"use strict";

var inky = require('../index.js');
var cheerio = require('../node_modules/cheerio');

describe("inky", function () {
  it("should be setting custom tags from object correctly", function () {

  inky.setTagArray();
  expect(inky.zfArray).toEqual([ 'button', 'row', 'callout', 'columns', 'subcolumns', 'container', 'inline-list-h', 'inline-list-v' ]);
  });

});
