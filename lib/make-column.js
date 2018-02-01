'use strict';

const format = require('util').format;
const $ = require('cheerio');
const getAttrs = require('./util/get-attrs');

/**
 * Returns output for column elements.
 * @todo This could be refactored to handle both cols and subcols.
 * @param {string} col - Column to format.
 * @returns {string} Column HTML.
 */
module.exports = function (col) {
  let output = '';
  const inner = $(col).html();
  let classes = [];
  let expander = '';
  const attrs = getAttrs(col);

  // Add 1 to include current column
  const colCount = $(col).siblings().length + 1;

  // Inherit classes from the <column> tag
  if ($(col).attr('class')) {
    classes = classes.concat($(col).attr('class').split(' '));
  }

  // Check for sizes. If no attribute is provided, default to small-12. Divide evenly for large columns
  const smallSize = $(col).attr('small') || this.columnCount;
  const largeSize = $(col).attr('large') || $(col).attr('small') || Math.floor(this.columnCount / colCount);
  const noExpander = $(col).attr('no-expander');

  classes.push(format('small-%s', smallSize));
  classes.push(format('large-%s', largeSize));

  // Add the basic "columns" class also
  classes.push('columns');

  // Determine if it's the first or last column, or both
  if ($(col).prev(this.components.columns).length === 0) {
    classes.push('first');
  }
  if ($(col).next(this.components.columns).length === 0) {
    classes.push('last');
  }

  // If the column contains a nested row, the .expander class should not be used
  if (
    parseInt(largeSize, 10) === this.columnCount &&
    col.find('.row, row').length === 0 &&
    (noExpander === undefined || noExpander === 'false')
  ) {
    expander = '\n<th class="expander"></th>';
  }

  // Final HTML output
  output = '<th class="%s" %s><table><tbody><tr><th>%s</th>%s</tr></tbody></table></th>';

  return format(output, classes.join(' '), attrs, inner, expander);
};
