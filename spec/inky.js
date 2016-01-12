"use strict";

var inky = require('../lib/inky');
var cheerio = require('cheerio');

describe("inky", function () {
  it("should be setting custom tags from object correctly", function () {

  inky.setTagArray();
  expect(inky.zfArray).toEqual([ 'button', 'row', 'callout', 'columns', 'subcolumns', 'container', 'inline-list-h', 'inline-list-v' ]);
  });

});
