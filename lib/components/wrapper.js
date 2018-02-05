'use strict';

const classnames = require('classnames');

module.exports = {
  name: 'Wrapper',
  props: {
    class: ''
  },
  render(props) {
    return `
      <table ${props.rest} class="${classnames('wrapper', props.class)}" align="center">
        <tbody>
          <tr>
            <td class="wrapper-inner">${props.children()}</td>
          </tr>
        </tbody>
      </table>
    `;
  }
};
