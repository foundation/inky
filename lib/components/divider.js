'use strict';

const cls = require('classnames');

module.exports = {
  name: 'Divider',
  props: {
    class: ''
  },
  render(element, props) {
    return `
      <table class="${cls('h-line', props.class)}">
        <tr>
          <th>&nbsp;</th>
        </tr>
      </table>
    `;
  }
};
