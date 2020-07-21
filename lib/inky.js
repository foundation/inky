var extend = require('util')._extend;
var values = require('object-values');
var cheerio = require('cheerio');

module.exports = Inky;

/**
 * Creates a new instance of the Inky parser.
 * @class
 */
function Inky(options) {
  options = options || {};

  this.cheerioOpts = options.cheerio;
  // HTML tags for custom components
  this.components = extend({
    button: 'button',
    row: 'row',
    columns: 'columns',
    container: 'container',
    callout: 'callout',
    inky: 'inky',
    blockGrid: 'block-grid',
    menu: 'menu',
    menuItem: 'item',
    center: 'center',
    spacer: 'spacer',
    wrapper: 'wrapper',
    hLine: 'h-line'
  }, options.components || {});

  // Column count for grid
  this.columnCount = options.columnCount || 12;

  this.componentTags = values(this.components);
}

/**
 * Awww yiss. Kickstarts the whole parser. Takes in HTML as a string, checks if there are any custom components. If there are, it replaces the nested components, traverses the DOM and replaces them with email markup.
 * @param {object} $ - Input HTML as a string
 * @returns {object} Modified HTML as a string
 */
Inky.prototype.releaseTheKraken = function(xmlString, cheerioOpts) {
  // This large compound selector looks for any custom tag loaded into Inky
  // <center> is an exception: the selector is center:not([data-parsed])
  // Otherwise the parser gets caught in an infinite loop where it continually tries to process the same <center> tags
  //
  // backwards compatible with old versions that pass in cheerio
  if(typeof(xmlString) !== 'string') {
    xmlString = xmlString.html();
  }
  var set = Inky.extractRaws(xmlString);
  var raws = set[0], string = set[1];
  var $ = cheerio.load(string, Inky.mergeCheerioOpts(cheerioOpts));
  var tags = this.componentTags.map(function(tag) {
    if (tag == 'center') {
      return tag + ':not([data-parsed])';
    }
    return tag;
  }).join(', ');

  // Because the structure of the DOM constantly shifts, we carefully go through each custom tag one at a time, until there are no more custom tags to parse
  while ($(tags).length > 0) {
    var elem = $(tags).eq(0);
    var newHtml = this.componentFactory(elem);
    elem.replaceWith(newHtml);
  }

  // Remove data-parsed attributes created for <center>
  $('[data-parsed]').removeAttr('data-parsed');

  string =  $.html();
  return Inky.reInjectRaws(string, raws);
}

Inky.mergeCheerioOpts = function(opts) {
  opts = opts || {};
  if(typeof(opts.decodeEntities) === 'undefined') {
    opts.decodeEntities = false;
  }
  return opts;
};

Inky.extractRaws = function(string) {
  var raws = [];
  var i = 0;
  var raw;
  var str = string
  var regex = /\< *raw *\>(.*?)\<\/ *raw *\>/i;
  while(raw = str.match(regex)) {
    raws[i] = raw[1];
    str = str.replace(regex, '###RAW' + i + '###');
    i = i+1;
  }
  return [raws, str];
};

Inky.reInjectRaws = function(string, raws) {
  var str = string;
  for (var i in raws) {
    str = str.replace('###RAW' + i + '###', raws[i])
  }
  return str;
};

Inky.prototype.componentFactory = require('./componentFactory');

Inky.prototype.makeColumn = require('./makeColumn');
