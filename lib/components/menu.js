'use strict';

const classnames = require('classnames');

module.exports = {
  name: 'Menu',
  props: {
    class: ''
  },
  render(element, props) {
    return `
      <table ${props.rest} class="${classnames('menu', props.class)}">
        <tbody>
          <tr>
            <td>
              <table>
                <tbody>
                  <tr>${props.children()}</tr>
                </tbody>
              </table>
            </td>
          </tr>
        </tbody>
      </table>
    `;
  }
};
