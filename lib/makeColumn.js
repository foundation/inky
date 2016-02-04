/**
 * Returns output for column elements.
 * @todo This could be refactored to handle both cols and subcols.
 * @param {object} $ - Instance of Cheerio.
 * @param {string} col - Column to format.
 * @param {string} type - Type of column.
 */
module.exports = function($, col, type) {
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
