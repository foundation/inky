'use strict';

const classnames = require('classnames');

module.exports = {
  name: 'Callout',
  props: {
    class: ''
  },
  render(element, props) {
    return `
      <table ${props.rest} class="callout">
        <tbody>
          <tr>
            <th class="${classnames('callout-inner', props.class)}">${props.children()}</th>
            <th class="expander"></th>
          </tr>
        </tbody>
      </table>
    `;
  }
};
