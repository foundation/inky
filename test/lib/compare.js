var Inky = require('../../lib/inky');
var cheerio = require('cheerio');
var htmlEqual = require('assert-html-equal');

module.exports = function compare(input, expected) {
  var inky = new Inky();
  var $ = cheerio.load(input);
  var output = inky.releaseTheKraken($).html();

  htmlEqual(output, expected);
}
