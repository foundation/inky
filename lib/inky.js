var cheerio = require('cheerio');

module.exports = Inky;

/**
 * Creates a new instance of the Inky parser.
 * @class
 */
function Inky () {
  this.components = {
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
  },

  // List of attributes we will not store as a class
  this.attributes = [
    'href'
  ],

  this.columnCount = 12;
}

/**
 * Sets a custom configuration for Inky.
 * @param {object} opts - Options to set.
 */
Inky.prototype.setConfig = function(opts) {
  for (var prop in opts) {
    this[prop] = opts[prop];
  }
}

/**
 * Awww yiss. Kickstarts the whole parser. Takes in HTML loaded via Cheerio as an argument, checks if there are any custom components. If there are, it replaces the nested components, traverses the DOM and replaces them with email markup.
 * @param {object} $ - Input HTML as a Cheerio object
 * @param {object} opts - Plugin configuration
 * @returns {object} Modified HTML as a Cheerio object
 */
Inky.prototype.releaseTheKraken = function($, opts) {
  var page = $.root().html(),
      self = this;

  // Set configuration
  if (opts) {
    self.setConfig(opts);
  }

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
  var self = this;

  // Create an array of our custom tags, if we haven't done so already
  if(!self.zfArray) {
    self.setTagArray();
  }

  // Check if the element is a custom element
  return self.zfArray.indexOf(elType) !== -1;
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

  // If array hasn't been set yet, set it with properties of object
  if (!self.zfArray) {
    self.setTagArray();
  }

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

  // If array hasn't been set yet, set it with properties of object
  if (!self.zfArray) {
    self.setTagArray();
  }

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

  // If array hasn't been set yet, set it with properties of object
  if (!self.zfArray) {
    self.setTagArray();
  }

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
 * Transcludes the attributes from our custom markup into the table markup
 * @param {object} component = Component as a Cheerio object.
 * @param {string} A string starting with the first class and other attributes following.
 */
Inky.prototype.addComponentAttrs = function(component) {
  var attributes = component.attr(),
      compAttrs  = {
        'class': '',
        'attributes': {}
      };

  for (var attr in attributes) {
    if (attr === 'class') {
      compAttrs.class += attributes[attr] + ' ';
    } else if (this.attributes.indexOf(attr) !== -1) {
      compAttrs.attributes[attr] = attributes[attr];
    } else {
      compAttrs.class += attr + '-' + attributes[attr] + ' ';
    }
  }

  compAttrs.class = compAttrs.class.replace(/\s$/, '');

  return compAttrs;
}

/**
 * Returns output for desired custom element
 * @param {object} $ - Instance of Cheerio.
 * @param {object} element - Element as a Cheerio object.
 * @param {string} type - Element type.
 * @returns {string} HTML converted from a custom element to table syntax.
 */
Inky.prototype.componentFactory = function($, element, type) {
  var output    = '',
      component = $(element),
      inner     = $(element).html(),
      compAttr = '',
      self      = this;

  if (component.attr() !== {}) {
   compAttr = self.addComponentAttrs(component);
  }

  switch (type) {
    case self.components.callout:
      if (component.parent() && self.isTableCell(component.parent()[0].name)) {
        output = '<table><tbody><tr><td class="callout' + compAttr.class +'">' + inner + '</td></tr></tbody></table>';
      }
      else {
        output = '<td class="callout ' + compAttr.class +'">' + inner + '</td>';
      }
      break;

    case self.components.button:
      // If we have the href attribute we can create an anchor for the inner of the button;
      if ('href' in compAttr.attributes) {
        inner = '<a href="' + compAttr.attributes.href + '">' + inner + '</a>';
      }

      // The .button class is always there, along with any others on the <button> element
      var classes = ['button'];
      if (compAttr.class.length) {
        classes = classes.concat(compAttr.class.split(' '));
      }

      output = '<table class="' + classes.join(' ') + '"><tr><td><table><tr><td>' + inner + '</td></tr></table></td></tr></table>';
      break;

    case self.components.subcolumns:
      output = self.makeColumn($, component, 'subcolumns');
      break;

    case self.components.container:
      output = '<table class="container' + compAttr.class + '"><tbody><tr><td>' + inner + '</td></tr></tbody></table>';
      break;

    case self.components.columns:
      output = self.makeColumn($, component, 'columns');
      break;

    case self.components.row:
      output = '<table class="row' + compAttr.class + '"><tbody><tr>'+ inner + '</tr></tbody></table>';
      break;

    case self.components.inky:
      output = '<tr><td><img src="https://raw.githubusercontent.com/arvida/emoji-cheat-sheet.com/master/public/graphics/emojis/octopus.png" /></tr></td>';
      break;

    case self.components.blockGrid:
      output = '<table class="block-grid up-' + component.attr('up') + '"><tr>' + inner + '</tr></table>';
      break;

    case self.components.menu:
      output = '<table class="menu"><tr>' + inner + '</tr></table>';
      break;

    case self.components.menuItem:
      output = '<td><a href="' + compAttr.attributes.href + '">' + inner + '</a></td>'
      break;

    default:
      // If it's not a custom component, return it as-is
      inner = $.html(element);
      output = '<tr><td>' + inner + '</td></tr>';
  };

  return output;
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

    // Put each child in its own <tr> unless it's a table element or a zfElement
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
