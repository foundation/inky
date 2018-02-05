'use strict';

const classnames = require('classnames');

module.exports = {
  name: 'Divider',
  props: {
    class: ''
  },
  render(props) {
    return `
      <table class="${classnames('h-line', props.class)}">
        <tr>
          <th>&nbsp;</th>
        </tr>
      </table>
    `;
  }
};
