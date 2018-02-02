'use strict';

const $ = require('cheerio');

module.exports = {
  name: 'Center',
  render(element, props) {
    if (element.children().length > 0) {
      element.children().each(function () {
        $(this).attr('align', 'center');
        $(this).addClass('float-center');
      });
      element.find('Item, .menu-item').addClass('float-center');
    }

    return `
      <center ${props.rest}>${props.children()}</center>
    `;
  }
};
