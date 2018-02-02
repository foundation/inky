/* eslint-disable complexity */

'use strict';

const format = require('util').format;
const $ = require('cheerio');
const getAttrs = require('./util/get-attrs');

/**
 * Returns output for desired custom element
 * @param {object} element - Element as a Cheerio object.
 * @returns {string} HTML converted from a custom element to table syntax.
 */
module.exports = function (element) {
  const fancyNewOutput = this.render(element);

  if (fancyNewOutput) {
    return fancyNewOutput;
  }

  const inner = element.html();
  const attrs = getAttrs(element);

  switch (element[0].name) {
    // <spacer>
    case this.components.spacer: {
      let classes = ['spacer'];
      let size;
      let html = '';
      if (element.attr('class')) {
        classes = classes.concat(element.attr('class').split(' '));
      }
      if (element.attr('size-sm') || element.attr('size-lg')) {
        if (element.attr('size-sm')) {
          size = (element.attr('size-sm'));
          html += format('<table %s class="%s hide-for-large"><tbody><tr><td height="' + size + 'px" style="font-size:' + size + 'px;line-height:' + size + 'px;">&nbsp;</td></tr></tbody></table>', attrs);
        }
        if (element.attr('size-lg')) {
          size = (element.attr('size-lg'));
          html += format('<table %s class="%s show-for-large"><tbody><tr><td height="' + size + 'px" style="font-size:' + size + 'px;line-height:' + size + 'px;">&nbsp;</td></tr></tbody></table>', attrs);
        }
      } else {
        size = (element.attr('size')) || 16;
        html += format('<table %s class="%s"><tbody><tr><td height="' + size + 'px" style="font-size:' + size + 'px;line-height:' + size + 'px;">&nbsp;</td></tr></tbody></table>', attrs);
      }

      if (element.attr('size-sm') && element.attr('size-lg')) {
        return format(html, classes.join(' '), classes.join(' '), inner);
      }

      return format(html, classes.join(' '), inner);
    }

    default:
      // If it's not a custom component, return it as-is
      return format('<tr><td>%s</td></tr>', $.html(element, this.cheerioOpts));
  }
};
