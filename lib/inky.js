var cheerio = require('cheerio');
var extend  = require('util')._extend;
var format  = require('util').format;

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
    callout: 'callout',
    columns: 'columns',
    subcolumns: 'subcolumns',
    container: 'container',
    inky: 'inky',
    blockGrid: 'block-grid',
    menu: 'menu',
    menuItem: 'item'
  }, options.components || {}),

  // List of attributes we will not store as a class
  this.attributes = options.attributes || ['href'];

  // Column count for grid
  this.columnCount = options.columnCount || 12;
}

/**
 * Awww yiss. Kickstarts the whole parser. Takes in HTML loaded via Cheerio as an argument, checks if there are any custom components. If there are, it replaces the nested components, traverses the DOM and replaces them with email markup.
 * @param {object} $ - Input HTML as a Cheerio object
 * @param {object} opts - Plugin configuration
 * @returns {object} Modified HTML as a Cheerio object
 */
Inky.prototype.releaseTheKraken = function($) {
  var page = $.root().html(),
      self = this;

  // Create an array of our custom tags
  self.setTagArray();

  // Find nested components
  if (self.checkZfComponents($) !== false) {
    var nestedComponents = self.findNestedComponents($, page);

    // Process each element to get the table markup
    $(nestedComponents).each(function(idx, el) {
      var containerScaffold = self.scaffoldElements($, $(el));
    });
  }

  return $;
}

/**
 * Sets the object property zfArray to an array containing the markup for our ZF custom elements.
 */
Inky.prototype.setTagArray = function() {
  var arr = [];
  var self = this;

  for (val in self.components) {
    arr.push(self.components[val]);
  }
  self.zfArray = arr;
}

/**
 * Checks if an element is a custom ZF element.
 * @param {string} elType - Tag name to check.
 * @returns {boolean} `true` if the tag is a custom element, `false` if not.
 */
Inky.prototype.isCustomElement = function(elType) {
  // Check if the element is a custom element
  return this.zfArray.indexOf(elType) !== -1;
}

/**
 * Checks if an element is an element with a td included. Currently it's a manual check. Array was populated from the markup from the component factory.
 * @param {string} elType - Tag name to check.
 * @returns {boolean} `true` if the element is a `td`, false if not.
 */
Inky.prototype.isTableCell = function(elType) {
  var tdEls = [this.components.subcolumns, this.components.callout, 'td'];

  // Check if the element is an element that comes with td
  return tdEls.indexOf(elType) > -1;
}

/**
 * Checks if an element is an element that is usually included with table markup.
 * @param {string} elType - Tag name to check.
 * @returns {boolean} `true` if the element is a `table`, `false` if not.
 */
Inky.prototype.isTable = function(elType) {
  var tableEls = ['td', 'tr', 'th', 'table', 'center', 'tbody'];

  // Check if the element is an element that comes with td
  return tableEls.indexOf(elType) > -1;
}

/**
 * Goes through array of custom nested components to determine whether or not there are any on the DOM
 * @param {object} $ - Input HTML as a Cheerio object
 * @returns {boolean} `true` if there are nested components in the DOM, or `false` otherwise.
 */
Inky.prototype.checkZfComponents = function($) {
  var self = this;

  $(self.zfArray).each(function(idx, zfElement) {
    // check if custom elements still exist
    if ($('center').find(zfElement).length > 0) {
      return true;
    }
  });
}

/**
 * Executes a function to find and return nested custom elements within another element.
 * @param {object} $ - Instance of Cheerio.
 * @param {string} str - HTML to check for nested components.
 * @returns {array} Names (i.e., tags) of the nested components found.
 */
Inky.prototype.findNestedComponents = function($, str) {
  var nestedComponents = [],
      self             = this,
      children;

  // If the nested component is an element, find the children.
  // NOTE: this is to avoid a Cheerio quirk where it will still pass special alphanumeric characters as a selector
  if (str.indexOf('<') !== -1) {
    children = $(str);
  };

  $(children).each(function(i, el) {
    // If the element's name matches an element in the array
    if (self.zfArray.indexOf(el.name) !== -1) {
      // Push them to array of nested component names
      nestedComponents.push(el.name);
    }
  });

  // Return array containing all nested components
  return nestedComponents;
}

/**
 * Executes a function to find and return deeply nested custom elements within another element. Uses the find selector rather than going through children.
 * @param {object} $ - Instance of Cheerio.
 * @param {string} el - String containing the markup of an element to be checked for nested components.
 * @returns {array} Names (i.e., tags) of nested components.
 */
Inky.prototype.findDeeplyNested = function($, el) {
  var nestedComponents = [],
      self             = this;

  // If the nested component is an element, find the children.
  // NOTE: this is to avoid a Cheerio quirk where it will still pass special alphanumeric characters as a selector
  if (el.indexOf('<') !== -1) {
    $(self.zfArray).each(function(idx, zfElement) {
      // find any nearby elements that are contained within el
      if ($(el).find(zfElement).length > 0) {
        nestedComponents.push(zfElement);
      }
    });
  };

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
  var output   = '',
      elMarkup = '',
      element  = $(str)[0],
      inner    = $(str).html(),
      self     = this;

  // Replace tags with proper table syntax
  // elMarkup retains the inner html within the markup
  if (element !== undefined) {
    elMarkup = self.componentFactory($, element, element.name);
    $(element).replaceWith(elMarkup);
  }
  else {
    return;
  }

  // Find if there are more nested elements in the inner syntax
  var moreNested = self.findNestedComponents($, inner);
  moreNested = moreNested.concat(self.findDeeplyNested($, inner));

  $(moreNested).each(function(idx, el) {
    // Call a recursively to replace all nested elements
    self.scaffoldElements($, $(el));
  });
}

/**
 * Returns output for desired custom element
 * @param {object} $ - Instance of Cheerio.
 * @param {object} element - Element as a Cheerio object.
 * @param {string} type - Element type.
 * @returns {string} HTML converted from a custom element to table syntax.
 */
Inky.prototype.componentFactory = function($, element, type) {
  var component = $(element),
      inner     = $(element).html(),
      self      = this;

  switch (type) {
    case self.components.callout:
      var classes = ['callout'];
      if (component.attr('class')) {
        classes = classes.concat(component.attr('class').split(' '));
      }

      if (component.parent() && self.isTableCell(component.parent()[0].name)) {
        return format('<table><tbody><tr><td class="%s">%s</td></tr></tbody></table>', classes.join(' '), inner);
      }
      else {
        return format('<td class="%s">%s</td>', classes.join(' '), inner);
      }

    case self.components.button:
      // If we have the href attribute we can create an anchor for the inner of the button;
      if (component.attr('href')) {
        inner = format('<a href="%s">%s</a>', component.attr('href'), inner);
      }

      // The .button class is always there, along with any others on the <button> element
      var classes = ['button'];
      if (component.attr('class')) {
        classes = classes.concat(component.attr('class').split(' '));
      }

      return format('<table class="%s"><tr><td><table><tr><td>%s</td></tr></table></td></tr></table>', classes.join(' '), inner);

    case self.components.subcolumns:
      return self.makeColumn($, component, 'subcolumns');

    case self.components.container:
      var classes = ['container'];
      if (component.attr('class')) {
        classes = classes.concat(component.attr('class').split(' '));
      }

      return format('<table class="%s"><tbody><tr><td>%s</td></tr></tbody></table>', classes.join(' '), inner);

    case self.components.columns:
      return self.makeColumn($, component, 'columns');

    case self.components.row:
      var classes = ['row'];
      if (component.attr('class')) {
        classes = classes.concat(component.attr('class').split(' '));
      }

      return format('<table class="%s"><tbody><tr>%s</tr></tbody></table>', classes.join(' '), inner);

    case self.components.inky:
      return '<tr><td><img src="https://raw.githubusercontent.com/arvida/emoji-cheat-sheet.com/master/public/graphics/emojis/octopus.png" /></tr></td>';

    case self.components.blockGrid:
      return format('<table class="block-grid up-%s"><tr>%s</tr></table>', component.attr('up'), inner);

    case self.components.menu:
      return format('<table class="menu"><tr>%s</tr></table>', inner);

    case self.components.menuItem:
      return format('<td><a href="%s">%s</a></td>', component.attr('href'), inner);

    default:
      // If it's not a custom component, return it as-is
      return format('<tr><td>%s</td></tr>', $.html(element));
  }
}

/**
 * Returns output for column elements.
 * @todo This could be refactored to handle both cols and subcols.
 * @param {object} $ - Instance of Cheerio.
 * @param {string} col - Column to format.
 * @param {string} type - Type of column.
 */
Inky.prototype.makeColumn = function($, col, type) {
  var output      = '',
      wrapperHTML = '',
      colSize     = '',
      colEl       = 'td',
      inner       = $(col).html(),
      colAttrs    = $(col).attr(),
      colClass    = colAttrs.class || '',
      self        = this,
      children;

  var classes = [];

  // Add 1 to include current column
  var colCount = $(col).siblings().length + 1;

  if ($(col).attr('el')) {
    colEl = $(col).attr('el');
  }

  if (colClass) {
    classes.push(colClass.split(' '));
  }

  // Check for sizes. If no attribute is provided, default to small-12. Divide evenly for large columns
  classes.push('small' + '-' + (colAttrs.small || self.columnCount));
  classes.push('large' + '-' + (colAttrs.large || colAttrs.small || Math.floor(self.columnCount/colCount)));

  // Start making markup
  if (type === 'columns') {
    classes.push('columns');

    // Determine if it's the first or last column, or both
    if (!$(col).prev(self.components.columns).length) classes.push('first');
    if (!$(col).next(self.components.columns).length) classes.push('last');

    output = '<th class="' + classes.join(' ') + '">';

    output += '<table><tr><th class="expander">';

    // If the nested component is an element, find the children
    // NOTE: this is to avoid a cheerio quirk where it will still pass special alphanumeric characters as a selector
    if (inner.indexOf('<') !== -1) {
      children = $(inner).nextUntil('columns');
    };

    // Put each child in its own <tr> unless it's a table element or a custom element
    $(children).each(function(idx, el) {
      if (el.name !== undefined && !self.isTable(el.name) && !self.isCustomElement(el.name)) {
        output += '<tr><td>' + $.html(el) + '</td><td class="expander"></td></tr>';
      }
      else {
        output += $.html(el) + '<td class="expander"></td>';
      }
    });

    output += '</th></tr></table></th>';
  }

  else if (type === 'subcolumns') {
    // If it is the last subcolumn, add the last class
    // With an extra check because the next item can be a td.expander
    if (!$(col).next(self.components.subcolumns)[0] && !$(col).next().next(self.components.subcolumns)[0]) {
      output = '<td class="sub-columns' + classes + '">' + inner + '</td>';
    }
    else {
      output = '<td class="sub-columns' + classes +'">' + inner + '</td>';
    }
  }
  else {
    return;
  }

  return output;
}
