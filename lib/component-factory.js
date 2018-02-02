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
    // <block-grid>
    case this.components.blockGrid: {
      let classes = ['block-grid', 'up-' + element.attr('up')];
      if (element.attr('class')) {
        classes = classes.concat(element.attr('class').split(' '));
      }
      return format('<table class="%s"><tbody><tr>%s</tr></tbody></table>', classes.join(' '), inner);
    }

    // <menu>
    case this.components.menu: {
      let classes = ['menu'];
      if (element.attr('class')) {
        classes = classes.concat(element.attr('class').split(' '));
      }
      return format('<table %s class="%s"><tbody><tr><td><table><tbody><tr>%s</tr></tbody></table></td></tr></tbody></table>', attrs, classes.join(' '), inner);
    }

    // <item>
    case this.components.menuItem: {
      // Prepare optional target attribute for the <a> element
      let target = '';
      if (element.attr('target')) {
        target = ' target=' + element.attr('target');
      }
      let classes = ['menu-item'];
      if (element.attr('class')) {
        classes = classes.concat(element.attr('class').split(' '));
      }
      return format('<th %s class="%s"><a href="%s"%s>%s</a></th>', attrs, classes.join(' '), element.attr('href'), target, inner);
    }

    // <callout>
    case this.components.callout: {
      let classes = ['callout-inner'];
      if (element.attr('class')) {
        classes = classes.concat(element.attr('class').split(' '));
      }

      return format('<table %s class="callout"><tbody><tr><th class="%s">%s</th><th class="expander"></th></tr></tbody></table>', attrs, classes.join(' '), inner);
    }

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

    // <wrapper>
    case this.components.wrapper: {
      let classes = ['wrapper'];
      if (element.attr('class')) {
        classes = classes.concat(element.attr('class').split(' '));
      }

      return format('<table %s class="%s" align="center"><tbody><tr><td class="wrapper-inner">%s</td></tr></tbody></table>', attrs, classes.join(' '), inner);
    }

    default:
      // If it's not a custom component, return it as-is
      return format('<tr><td>%s</td></tr>', $.html(element, this.cheerioOpts));
  }
};
