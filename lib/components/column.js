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
  render(props, element, opts) {
    const colCount = element.siblings().length + 1; // `.sliblings()` doesn't include the column itself, so add 1
    const smallSize = props.small || opts.columnCount;
    const largeSize = props.large || props.small || Math.floor(opts.columnCount / colCount);
    const expand =
      parseInt(largeSize, 10) === opts.columnCount && // Column is full width
      element.find('.row, Row').length === 0 && // No nested grid inside column
      props['no-expander'] === false; // `no-expander` is not set on column

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
