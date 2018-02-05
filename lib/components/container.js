'use strict';

const classnames = require('classnames');

module.exports = {
  name: 'Container',
  props: {
    class: ''
  },
  render(props) {
    return `
      <table ${props.rest} align="center" class="${classnames('container', props.class)}">
        <tbody>
          <tr>
            <td>${props.children()}</td>
          </tr>
        </tbody>
      </table>
    `;
  }
};
