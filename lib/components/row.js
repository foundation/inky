'use strict';

const classnames = require('classnames');

module.exports = {
  name: 'Row',
  props: {
    class: ''
  },
  render(props) {
    return `
      <table ${props.rest} class="${classnames('row', props.class)}">
        <tbody>
          <tr>${props.children()}</tr>
        </tbody>
      </table>
    `;
  }
};
