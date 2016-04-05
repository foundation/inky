var format = require('util').format;
var $ = require('cheerio');

var getKeyByValue = function(obj, val) {
  for (var key in obj) {
    if (obj[key] === val) {
      return key;
    }
  }
  return false;
};

/**
 * Returns output for desired custom element
 * @param {object} element - Element as a Cheerio object.
 * @returns {string} HTML converted from a custom element to table syntax.
 */
module.exports = function(element) {
  var tag = element[0].name;
  var componentKey = getKeyByValue(this.components, tag);

  if (componentKey && this.componentTags.indexOf(tag) > -1 && typeof this.componentLibrary[componentKey] === 'function') {
    return this.componentLibrary[componentKey].call(this, element);
  } else {
    // If it's not a custom component, return it as-is
    return format('<tr><td>%s</td></tr>', $.html(element));
  }
}
