var cheerio = require('cheerio');

module.exports = Inky;

/**
 * Creates a new instance of the Inky parser.
 * @class
 */
function Inky () {
  this.zfTags = {
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

  this.grid = 12;
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
  var center = $.root().html(),
      self   = this;

  // set configuration
  if (opts) {
    self.setConfig(opts);
  }
  // create an array of our custom tags
  self.setTagArray();

  //find nested components
  if (self.checkZfComponents($) !== false) {
    var nestedComponents = self.findNestedComponents($, center);
    // process each element to get the table markup
    $(nestedComponents).each(function(idx, el) {
      var containerScaffold = self.scaffoldElements($, $(el));
    });

    // remove any blank spaces between classes
    // and reload into cheerio
    str = self.removeBlankSpaces($.html());
    $ = cheerio.load(str);

    // see the mark up for dev purposes
    // console.log($.html());
  }
  else {
    console.log("all done");
  }
  return $;
}


/**
 * Returns a list of custom Inky tags.
 * @returns {object} List of tags.
 */
Inky.prototype.getTags = function() {
  return this.zfTags;
}

/**
 * Sets the object property zfArray to an array containing the markup for our ZF custom elements
 */
Inky.prototype.setTagArray = function() {
  var arr = [];
  var self = this;

  for (val in self.zfTags) {
    arr.push(self.zfTags[val]);
  }
  self.zfArray = arr;
}

/**
 * Checks if an element is a custom ZF element.
 * @param {string} elType - Tag name to check.
 * @returns {boolean} `true` if the tag is a custom element, `false` if not.
 */
Inky.prototype.isZfElement = function(elType) {
  var self = this;
  // create an array of our custom tags, if we haven't done so already
  if(!self.zfArray) {
    self.setTagArray();
  }

  // if the element is a custom element
  if (self.zfArray.indexOf(elType) !== -1) {
    // return true
    return true;
  }
  else {
    return false;
  }
}

/**
 * Checks if an element is an element with a td included. Currently it's a manual check. Array was populated from the markup from the component factory.
 * @param {string} elType - Tag name to check.
 * @returns {boolean} `true` if the element is a `td`, false if not.
 */
Inky.prototype.isTdElement = function(elType) {
  var tdEls = [this.zfTags.subcolumns, this.zfTags.callout, 'td'];

  // if the element is an element that comes with td
  if (tdEls.indexOf(elType) > -1) {
    // return true
    return true;
  }
  else {
    return false;
  }
}

/**
 * Checks if an element is an element that is usually included with table markup.
 * @param {string} elType - Tag name to check.
 * @returns {boolean} `true` if the element is a `table`, `false` if not.
 */
Inky.prototype.isTableElement = function(elType) {
  var tableEls = ['td', 'tr', 'th', 'table', 'center', 'tbody'];

  // if the element is an element that comes with td
  if (tableEls.indexOf(elType) > -1) {
    // return true
    return true;
  }
  else {
    return false;
  }
}

/**
 * Executes a function place the correct mark up for custom components in the correct place in the DOM. It is a recursive function that drills down the DOM to find all custom nested elements within an element and replaces the custom tags with the correct table email markup. I got a blank space, baby.
 * @param {string} str - String to replace.
 * @return (string) The modified string without blank spaces.
 */
Inky.prototype.removeBlankSpaces = function(str) {
  // remove any blank spaces between classes we may have put in
  str = str.replace( / "+/g, '"' );
  // str = str.replace( /" /+/g, '' );
  return str;
}

/**
 * Goes through array of custom nested components to determine whether or not there are any on the DOM
 * @param {object} $ - Input HTML as a Cheerio object
 * @returns {boolean} `true` if there are nested components in the DOM, or `false` otherwise.
 */
Inky.prototype.checkZfComponents = function($) {
  var self = this;

  // if array hasn't been set yet, set it with properties of object
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

  // if array hasn't been set yet, set it with properties of object
  if (!self.zfArray) {
    self.setTagArray();
  }
  // if the nested component is an element, find the children
  // NOTE: this is to avoid a cheerio quirk where it will still pass
  // special alphanumeric characters as a selector

  if (str.indexOf('<') !== -1) {
    children = $(str);
  };

  $(children).each(function(i, el) {

    // if the element's name matches an element in the array
    if (self.zfArray.indexOf(el.name) !== -1) {
      // push them to array of nested component names
      nestedComponents.push(el.name);
    }

    // // if the element's name matches an element in the array
    // var basics = ['p', 'h1'];
    // if (el.name !== undefined && basics.indexOf(el.name) > -1) {
    // //   // push them to array of nested component names
    //   nestedComponents.push(el.name);
    // }
  });
  // return array containing all nested components
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

  // if array hasn't been set yet, set it with properties of object
  if (!self.zfArray) {
    self.setTagArray();
  }

  // if the nested component is an element, find the children
  // NOTE: this is to avoid a cheerio quirk where it will still pass
  // special alphanumeric characters as a selector
  if (el.indexOf('<') !== -1) {
    $(self.zfArray).each(function(idx, zfElement) {
      // find any nearby elements that are contained within el
      if ($(el).find(zfElement).length > 0) {
        nestedComponents.push(zfElement);
      }
    });
  };

  // return array containing all nested components
  return nestedComponents;
}

/**
 * Executes a function place the correct mark up for custom components in the correct place in the DOM. It is a recursive function that drills down the DOM to find all custom nested elements within an element and replaces the custom tags with the correct table email markup.
 * @param {object} $ - Instance of Cheerio
 * @param {string} str - Markup of a single element.
 */
Inky.prototype.scaffoldElements = function($, str) {
  // take inner html of elements and nest them inside each others
  var output   = '',
      elMarkup = '',
      element  = $(str)[0],
      inner    = $(str).html(),
      self     = this;

  // replace tags with proper table syntax
  // elMarkup retains the inner html within the markup
  if (element !== undefined) {
    elMarkup = self.componentFactory($, element, element.name);
    $(element).replaceWith(elMarkup);
  }
  else {
    return;
  }

  // find if there are more nested elements in the inner syntax
  var moreNested = self.findNestedComponents($, inner);
  moreNested = moreNested.concat(self.findDeeplyNested($, inner));


  $(moreNested).each(function(idx, el) {
    // call a recursion to replace all nested elements
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
        'class': ' ',
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
    case self.zfTags.callout:
      if (component.parent() && self.isTdElement(component.parent()[0].name)) {
        output = '<table><tbody><tr><td class="callout' + compAttr.class +'">' + inner + '</td></tr></tbody></table>';
      }
      else {
        output = '<td class="callout ' + compAttr.class +'">' + inner + '</td>';
      }
      break;

    case self.zfTags.button:
      // If we have the href attribute we can create an anchor for the inner of the button;
      if ('href' in compAttr.attributes) {
        inner = '<a href="' + compAttr.attributes.href + '">' + inner + '</a>';
      }
      // if parent is a callout, you don't need the tds
      // if (component.parent() && self.isTdElement(component.parent()[0].name)) {
        // output = '<table class="button' + compAttr.class +'"<tr><td>' + inner + '</td></tr></table>';
      // }
      // else {
        output = '<table class="button' + compAttr.class +'"><tr><td><table><tr><td>' + inner + '</td></tr></table></td></tr></table>';
      // }
      break;

    case self.zfTags.subcolumns:
      output = self.makeCols($, component, 'subcolumns');
      break;

    case self.zfTags.container:
      output = '<table class="container' + compAttr.class + '"><tbody><tr><td>' + inner + '</td></tr></tbody></table>';
      break;

    case self.zfTags.columns:
      output = self.makeCols($, component, 'columns');
      break;

    case self.zfTags.row:
      output = '<table class="row' + compAttr.class + '"><tbody><tr>'+ inner + '</tr></tbody></table>';
      break;

    case self.zfTags.inky:
      output = '<tr><td><img src="https://raw.githubusercontent.com/arvida/emoji-cheat-sheet.com/master/public/graphics/emojis/octopus.png" /></tr></td>';
      break;

    case self.zfTags.blockGrid:
      output = '<table class="block-grid up-' + component.attr('up') + '"><tr>' + inner + '</tr></table>';
      break;

    case self.zfTags.menu:
      output = '<table class="menu"><tr>' + inner + '</tr></table>';
      break;

    case self.zfTags.menuItem:
      output = '<td><a href="' + compAttr.attributes.href + '">' + inner + '</a></td>'
      break;

    default:
      // unless it's a special element, just grab the inside
      // another cheerio quirk
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
Inky.prototype.makeCols = function($, col, type) {
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

  // check for sizes
  // if no attribute is provided, default to small-12
  // divide evenly for large columns
  classes.push('small' + '-' + (colAttrs.small || self.grid));
  classes.push('large' + '-' + (colAttrs.large || colAttrs.small || Math.floor(self.grid/colCount)));

  // start making markup
  if (type === 'columns') {
    classes.push('columns');

    // Determine if it's the first or last column, or both
    if (!$(col).prev(self.zfTags.columns).length) classes.push('first');
    if (!$(col).next(self.zfTags.columns).length) classes.push('last');

    output = '<th class="' + classes.join(' ') + '">';

    output += '<table><tr><th class="expander">';

    // if the nested component is an element, find the children
    // NOTE: this is to avoid a cheerio quirk where it will still pass
    // special alphanumeric characters as a selector
    if (inner.indexOf('<') !== -1) {
      children = $(inner).nextUntil('columns');
    };

    // put each child in its own tr
    // unless it's a table element or a zfElement
    $(children).each(function(idx, el) {
      if (el.name !== undefined && !self.isTableElement(el.name) && !self.isZfElement(el.name)) {
        output += '<tr><td>' + $.html(el) + '</td><td class="expander"></td></tr>';
      }
      else {
        output += $.html(el) + '<td class="expander"></td>';
      }
    });

    output += '</th></tr></table></th>';
  }

  else if (type === 'subcolumns') {
    // if it is the last subcolumn, add the last class
    // With an extra check because the next item can be a td.expander
    if (!$(col).next(self.zfTags.subcolumns)[0] && !$(col).next().next(self.zfTags.subcolumns)[0]) {
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
