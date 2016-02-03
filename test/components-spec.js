"use strict";

var Inky = require('../lib/inky');
var cheerio = require('cheerio');
var assert = require('assert');

describe("the components", function () {
  xit("returns basic button syntax", function () {
    var inky = new Inky();
    var $ = cheerio.load('<center><button class="inky" href="http://foundation.zurb.com/emails.html">Big button</button></center>');

    $ = inky.releaseTheKraken($);
    assert.equal($.html(), '<center><td><table class="button inky"><tbody><tr><td><a href="http://foundation.zurb.com/emails.html">Big button</a></td></tr></tbody></table></td></center>');
  });
});
