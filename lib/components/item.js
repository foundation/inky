'use strict';

const classnames = require('classnames');
const iff = require('../iff');

module.exports = {
  name: 'Item',
  props: {
    class: '',
    href: false,
    target: false
  },
  render(props) {
    const anchor = `<a href="${props.href}"${iff(props.target, ` target="${props.target}"`)}>${props.children()}</a>`;

    return `
      <th ${props.rest} class="${classnames('menu-item', props.class)}">${anchor}</th>
    `;
  }
};
