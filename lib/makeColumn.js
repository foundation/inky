var format = require('util').format;
var multiline = require('multiline');
var $ = require('cheerio');

/**
 * Returns output for column elements.
 * @todo This could be refactored to handle both cols and subcols.
 * @param {string} col - Column to format.
 * @returns {string} Column HTML.
 */
module.exports = function(col) {
  var output  = '';
  var inner   = $(col).html();
  var classes = [];
  var thClass = '';

  // Add 1 to include current column
  var colCount = $(col).siblings().length + 1;

  // Inherit classes from the <column> tag
  if ($(col).attr('class')) {
    classes.push($(col).attr('class').split(' '));
  }

  // Check for sizes. If no attribute is provided, default to small-12. Divide evenly for large columns
  classes.push(format('small-%s', $(col).attr('small') || this.columnCount));
  classes.push(format('large-%s', $(col).attr('large') || $(col).attr('small') || Math.floor(this.columnCount/colCount)));

  // Add the basic "columns" class also
  classes.push('columns');

  // Determine if it's the first or last column, or both
  if (!$(col).prev(this.components.columns).length) classes.push('first');
  if (!$(col).next(this.components.columns).length) classes.push('last');

  // If the column contains a nested row, the .expander class should not be used
  if (!col.find('.row, row').length) {
    thClass = ' class="expander"';
  }

  // Final HTML output
  output = multiline(function() {/*
    <th class="%s">
      <table>
        <tr>
          <th%s>%s</th>
        </tr>
      </table>
    </th>
  */});

  return format(output, classes.join(' '), thClass, inner);
}
