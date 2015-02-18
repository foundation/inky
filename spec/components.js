/* global describe, it, expect */

"use strict";

var inky = require('../index.js');
var cheerio = require('../node_modules/cheerio');

describe("the components", function () {

  it("returns basic button syntax", function () {
    var $ = cheerio.load('<center><row><columns large="12"><button>Big button</button></columns></row></center>');
    
    $ = inky.releaseTheKraken($);
    expect($.html()).toEqual('<center><table class="row"><tbody><tr><td class="wrapper"><table class="small-12 large-12 columns"><tr><table class="button"><tbody><tr><td>Big button</td></tr></tbody></table><td class="expander"></td></tr></table></td></tr></tbody></table></center>');
  });


});
