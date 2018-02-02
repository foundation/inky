'use strict';

const classnames = require('classnames');
const iff = require('../util/iff');

const expander = '\n<td class="expander"></td>';

module.exports = {
  name: 'Button',
  props: {
    class: '',
    href: false
  },
  render(element, props) {
    const expanded = props.class.match(/expand(ed)?/) !== null;
    const wrapper = children => expanded ? `<Center>${children}</Center>` : children;
    const anchor = children => props.href ? `
      <a href="${props.href}" ${props.rest}>${children}</a>
    `.trim() : children;

    return `
      <table class="${classnames('button', props.class)}">
        <tbody>
          <tr>
            <td>
              <table>
                <tbody>
                  <tr>
                    <td>${wrapper(anchor(props.children()))}</td>
                  </tr>
                </tbody>
              </table>
            </td>${iff(expanded, expander)}
          </tr>
        </tbody>
      </table>
    `;
  }
};
