var format = require('util').format;
var $ = require('cheerio');

/**
 * Returns output for desired custom element
 * @param {object} element - Element as a Cheerio object.
 * @returns {string} HTML converted from a custom element to table syntax.
 */
module.exports = function(element) {
  if (this.components[element[0].name] && this.componentLibrary[element[0].name]) {
    return this.componentLibrary[element[0].name].call(this, element);
  } else {
    // If it's not a custom component, return it as-is
    return format('<tr><td>%s</td></tr>', $.html(element));
  }
}
