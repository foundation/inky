/* global describe, it, expect */

"use strict";

var inky = require('../index.js');
var cheerio = require('../node_modules/cheerio');

describe("the grid", function () {
  // it("returns the correct row syntax", function() {
  //   var $ = cheerio.load('<center><row>This is a row!</row></center>');
    
  //   $ = inky.releaseTheKraken($);
  //   expect($.html()).toEqual('<center><table class="row"><tbody><tr>This is a row!</tr></tbody></table></center>');
  // })

  // it("returns the correct container syntax", function() {
  //   var $ = cheerio.load('<center><container>This is a container!</container></center>');
    
  //   $ = inky.releaseTheKraken($);
  //   expect($.html()).toEqual('<center><table class="container"><tbody><tr><td>This is a container!</td></tr></tbody></table></center>');
  // })

  // it("returns the correct column syntax", function () {
  //   var $ = cheerio.load('<center><row><columns large="10"><h3>This is 10 Columns</h3></columns><columns large="2"><h3>This is 2 Columns</h3></columns></row></center>');
    
  //   $ = inky.releaseTheKraken($);
  //   expect($.html()).toEqual('<center><table class="row"><tbody><tr><td class="wrapper"><table class="small-12 large-10 columns"><tr><td><h3>This is 10 Columns</h3></td><td class="expander"></td></tr></table></td><td class="wrapper last"><table class="small-12 large-2 columns"><tr><td><h3>This is 2 Columns</h3></td><td class="expander"></td></tr></table></td></tr></tbody></table></center>');
  // });

  it("nests subcolumns correctly", function() {
    var $ = cheerio.load('<center><row><columns large="6" small="12"><subcolumns large="4"><h3>Sub column 1</h3></subcolumns><subcolumns large="8"><h3>Sub column 2</h3></subcolumns></columns><columns large="6" small="12"><subcolumns large="6"><h3>Sub column 1.1</h3></subcolumns><subcolumns large="6"><h3>Sub column 1.2</h3></subcolumns></columns></row></center>');

    $ = inky.releaseTheKraken($);
    expect($.html()).toEqual('<center><table class="row"><tbody><tr><td class="wrapper"><table class="small-12 large-6 columns"><td class="sub-columns small-12 large-4"><h3>Sub column 1</h3></td><td class="expander"></td><td class="sub-columns last small-12 large-8"><h3>Sub column 2</h3></td><td class="expander"></td></table></td><td class="wrapper last"><table class="small-12 large-6 columns"><td class="sub-columns last small-12 large-6"><h3>Sub column 1.1</h3></td><td class="expander"></td><td class="sub-columns last small-12 large-6"><h3>Sub column 1.2</h3></td><td class="expander"></td></table></td></tr></tbody></table></center>');
  });

  // it("nests rows in containers", function() {
  //   var $ = cheerio.load('<center><container></row>Row in container</row></container></center>');

  //   $ = inky.releaseTheKraken($);
  //   expect($.html()).toEqual('<center><table class="container"><tbody><tr><td>Row in container</td></tr></tbody></table></center>');
  // })

  // it("will place the 'last' class on the last column", function() {
  //   var $ = cheerio.load('<center><row><columns large="10">.ten cols</columns><columns large="2">.two cols</columns></row></center>');

  //   $ = inky.releaseTheKraken($);
  //   expect($.html()).toEqual('<center><table class="row"><tbody><tr><td class="wrapper"><table class="small-12 large-10 columns"><tr>.ten cols<td class="expander"></td></tr></table></td><td class="wrapper"><table class="small-12 large-2 columns"><tr>.two cols<td class="expander"></td></tr></table></td></tr></tbody></table></center>');

  // })

  // it("will place the 'last' class on a solo column", function() {
  //   var $ = cheerio.load('<center><row><columns large="12">forever alone</columns></row></center>');

  //   $ = inky.releaseTheKraken($);
  //   expect($.html()).toEqual('<center><table class="row"><tbody><tr><td class="wrapper"><table class="small-12 large-12 columns"><tr>forever alone<td class="expander"></td></tr></table></td></tr></tbody></table></center>');

  // })

  // it("handles deeply nested components", function() {
  //   var $ = cheerio.load('<center><row><columns large="12"><p><callout>deep stuff</callout></p></columns></row></center>');

  //   $ = inky.releaseTheKraken($);
  //   expect($.html()).toEqual('<center><table class="row"><tbody><tr><td class="wrapper"><table class="small-12 large-12 columns"><tr><p><td class="callout">deep stuff</td></p><td class="expander"></td></tr></table></td></tr></tbody></table></center>');

  // })

  // it("automatically assigns large columns if no large attribute is assigned", function() {
  //   var $ = cheerio.load('<center><row><columns small="12"></columns><columns small="12"></columns><columns small="12"></columns></row></center>');

  //   $ = inky.releaseTheKraken($);
  //   expect($.html()).toEqual('<center><table class="row"><tbody><tr><td class="wrapper"><table class="small-12 large-4 columns"><tr><td class="expander"></td></tr></table></td><td class="wrapper"><table class="small-12 large-4 columns"><tr><td class="expander"></td></tr></table></td><td class="wrapper"><table class="small-12 large-4 columns"><tr><td class="expander"></td></tr></table></td></tr></tbody></table></center>');

  // })

  // it("automatically assigns large columns if no large attribute is assigned, with custom grids", function() {
  //   var $ = cheerio.load('<center><row><columns></columns><columns></columns><columns></columns></row></center>');

  //   var opts = {
  //     grid: 15
  //   }
  //   $ = inky.releaseTheKraken($, opts);
  //   expect($.html()).toEqual('<center><table class="row"><tbody><tr><td class="wrapper"><table class="small-15 large-5 columns"><tr><td class="expander"></td></tr></table></td><td class="wrapper"><table class="small-15 large-5 columns"><tr><td class="expander"></td></tr></table></td><td class="wrapper"><table class="small-15 large-5 columns"><tr><td class="expander"></td></tr></table></td></tr></tbody></table></center>');

  // })

  // it("automatically assigns small columns as full width, with custom grids", function() {
  //   var $ = cheerio.load('<center><row><columns></columns><columns></columns><columns></columns></row></center>');

  //   var opts = {
  //     grid: 15
  //   }
  //   $ = inky.releaseTheKraken($, opts);
  //   expect($.html()).toEqual('<center><table class="row"><tbody><tr><td class="wrapper"><table class="small-15 large-5 columns"><tr><td class="expander"></td></tr></table></td><td class="wrapper"><table class="small-15 large-5 columns"><tr><td class="expander"></td></tr></table></td><td class="wrapper"><table class="small-15 large-5 columns"><tr><td class="expander"></td></tr></table></td></tr></tbody></table></center>');

  // })
});
