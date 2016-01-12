/* global describe, it, expect */

"use strict";

var inky = require('../index.js');
var cheerio = require('../node_modules/cheerio');

describe("the components", function () {

  it("returns basic button syntax", function () {
    var $ = cheerio.load('<center><button class="inky" href="http://foundation.zurb.com/emails.html">Big button</button></center>');
    
    $ = inky.releaseTheKraken($);
    expect($.html()).toEqual('<center><td><table class="button inky"><tbody><tr><td><a href="http://foundation.zurb.com/emails.html">Big button</a></td></tr></tbody></table></td></center>');
  });
});

