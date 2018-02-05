'use strict';

const classnames = require('classnames');

module.exports = {
  name: 'BlockGrid',
  props: {
    class: '',
    up: 0
  },
  render(props) {
    return `
      <table class="${classnames(`block-grid up-${props.up}`, props.class)}" ${props.rest}>
        <tbody>
          <tr>${props.children()}</tr>
        </tbody>
      </table>
    `;
  }
};
