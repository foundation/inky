/* eslint-disable complexity */

'use strict';

const format = require('util').format;
const $ = require('cheerio');

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

  return format('<tr><td>%s</td></tr>', $.html(element, this.cheerioOpts));
};
