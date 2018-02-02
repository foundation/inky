'use strict';

const classnames = require('classnames');

const spacer = (props, size, state) => `
  <table ${props.rest} class="${classnames('spacer', props.class, state && `${state}-for-large`)}">
    <tbody>
      <tr>
        <td height="${size}px" style="font-size:${size}px;line-height:${size}px;">&nbsp;</td>
      </tr>
    </tbody>
  </table>
`;

module.exports = {
  name: 'Spacer',
  props: {
    class: '',
    size: 16,
    'size-sm': null,
    'size-lg': null
  },
  render(element, props) {
    const output = ['sm', 'lg'].reduce((html, val) => {
      const size = props[`size-${val}`];

      if (size) {
        const state = val === 'sm' ? 'hide' : 'show';
        return html + spacer(props, size, state).trim();
      }

      return html;
    }, '');

    if (output.length === 0) {
      return spacer(props, props.size);
    }

    return output;
  }
};
