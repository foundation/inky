'use strict';

const classnames = require('classnames');
const iff = require('../iff');

const expander = '\n<th class="expander"></th>';

module.exports = {
  name: 'Column',
  props: {
    class: '',
    small: null,
    large: null,
    'no-expander': false
  },
  render(element, props, opts) {
    // Add 1 to include current column
    const colCount = element.siblings().length + 1;
    const smallSize = element.attr('small') || opts.columnCount;
    const largeSize = element.attr('large') || element.attr('small') || Math.floor(opts.columnCount / colCount);
    const expand =
      parseInt(largeSize, 10) === opts.columnCount &&
      element.find('.row, Row').length === 0 &&
      props['no-expander'] === false;

    const classes = classnames(props.class, `small-${smallSize} large-${largeSize} columns`, {
      first: element.prev('.column, Column').length === 0,
      last: element.next('.column, Column').length === 0
    });

    return `
      <th class="${classes}" ${props.rest}>
        <table>
          <tbody>
            <tr>
              <th>${props.children()}</th>${iff(expand, expander)}
            </tr>
          </tbody>
        </table>
      </th>
    `;
  }
};
