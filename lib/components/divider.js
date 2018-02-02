'use strict';

const cls = require('classnames');

module.exports = {
  name: 'Divider',
  class: ['h-line'],
  render(element, props) {
    return `
      <table class="${cls(props.class)}">
        <tr>
          <th>&nbsp;</th>
        </tr>
      </table>
    `;
  }
};
