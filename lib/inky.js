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
    inky: 'inky',
    blockGrid: 'block-grid',
    menu: 'menu',
    menuItem: 'item'
  }, options.components || {}),

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
  var page = $.root().html();

  // Only run parser if special components can be found
  if (this.componentsExist($)) {
    var nestedComponents = this.findNestedComponents($, page);

    // Process each element to get the table markup
    $(nestedComponents).each(function(i, el) {
      var containerScaffold = this.scaffoldElements($, $(el));
    }.bind(this));
  }

  return $;
}

/**
 * Checks the input HTML to determine if custom Inky elements exist inside of it. This function is used to skip parsing entirely if there aren't any.
 * @param {object} $ - Input HTML as a Cheerio object
 * @returns {boolean} `true` if there are nested components in the DOM, or `false` otherwise.
 */
Inky.prototype.componentsExist = function($) {
  return $(this.componentTags.join(', ')).length ? true : false;
}

/**
 * Executes a function to find and return nested custom elements within another element.
 * @param {object} $ - Instance of Cheerio.
 * @param {string} str - HTML to check for nested components.
 * @returns {array} Names (i.e., tags) of the nested components found.
 */
Inky.prototype.findNestedComponents = function($, str) {
  var nestedComponents = [];
  var children;

  // If the nested component is an element, find the children.
  // NOTE: this is to avoid a Cheerio quirk where it will still pass special alphanumeric characters as a selector
  if (str.indexOf('<') !== -1) {
    children = cheerio(str);
  }

  $(children).each(function(i, el) {
    // If the element's name matches an element in the array
    if (this.componentTags.indexOf(el.name) !== -1) {
      // Push them to array of nested component names
      nestedComponents.push(el.name);
    }
    else if (el.name != undefined && el.name != "!doctype") {
      nestedComponents = nestedComponents.concat(this.findNestedComponents($, $(el).html()));
    }
  }.bind(this));

  // Return array containing all nested components
  return nestedComponents;
}

/**
 * Executes a function place the correct mark up for custom components in the correct place in the DOM. It is a recursive function that drills down the DOM to find all custom nested elements within an element and replaces the custom tags with the correct table email markup.
 * @param {object} $ - Instance of Cheerio
 * @param {string} str - Markup of a single element.
 */
Inky.prototype.scaffoldElements = function($, str) {
  // Take inner html of elements and nest them inside each others
  var output   = '';
  var elMarkup = '';
  var element  = $(str)[0];
  var inner    = $(str).html();

  // Replace tags with proper table syntax
  // elMarkup retains the inner html within the markup
  if (element !== undefined) {
    elMarkup = this.componentFactory(element);
    $(element).replaceWith(elMarkup);
  }
  else {
    return;
  }

  // Find if there are more nested elements in the inner syntax
  var moreNested = this.findNestedComponents($, inner);

  $(moreNested).each(function(i, el) {
    // Call recursively to replace all nested elements
    this.scaffoldElements($, $(el));
  }.bind(this));
}

Inky.prototype.componentFactory = require('./componentFactory');

Inky.prototype.makeColumn = require('./makeColumn');
