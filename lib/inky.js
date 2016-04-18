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
    wrapper: 'wrapper'
  }, options.components || {});

  // Column count for grid
  this.columnCount = options.columnCount || 12;

  this.componentTags = values(this.components);
}

/**
 * Awww yiss. Kickstarts the whole parser. Takes in HTML loaded via Cheerio as an argument, checks if there are any custom components. If there are, it replaces the nested components, traverses the DOM and replaces them with email markup.
 * @param {object} $ - Input HTML as a Cheerio object
 * @returns {object} Modified HTML as a Cheerio object
 */
Inky.prototype.releaseTheKraken = function($) {
  // This large compound selector looks for any custom tag loaded into Inky
  // <center> is an exception: the selector is center:not([data-parsed])
  // Otherwise the parser gets caught in an infinite loop where it continually tries to process the same <center> tags
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

  return $;
}

Inky.prototype.componentFactory = require('./componentFactory');

Inky.prototype.makeColumn = require('./makeColumn');
