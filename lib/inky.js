'use strict';

const cheerio = require('cheerio');
const requireDir = require('require-dir');
const deepAssign = require('deep-assign');
const renderer = require('./renderer');
const raw = require('./raw');

/**
 * Creates a new instance of the Inky parser.
 */
module.exports = class Inky {
  constructor(opts) {
    const components = requireDir('./components');

    this.options = deepAssign({
      cheerio: {
        decodeEntities: false,
        lowerCaseTags: false
      },
      columnCount: 12
    }, opts || {});

    this.library = new Map(
      Object.keys(components).map(k => [components[k].name, components[k]])
    );

    this.selectors = Array.from(this.library.keys()).join(', ');

    this.render = renderer(this.library, this.options);
  }

  /**
   * Awww yiss. Kickstarts the whole parser. Takes in HTML as a string, checks if there are any custom components. If there are, it replaces the nested components, traverses the DOM and replaces them with email markup.
   * @param {object} $ - Input HTML as a string
   * @returns {object} Modified HTML as a string
   */
  releaseTheKraken(xmlString, cheerioOpts) {
    if (typeof (xmlString) !== 'string') {
      xmlString = xmlString.html();
    }

    const set = raw.extract(xmlString);
    const raws = set[0];
    const string = set[1];
    const $ = cheerio.load(string, Object.assign({}, this.options.cheerio, cheerioOpts));

    // Because the structure of the DOM constantly shifts, we carefully go through each custom tag one at a time, until there are no more custom tags to parse
    while ($(this.selectors).length > 0) {
      const elem = $(this.selectors).eq(0);
      const newHtml = this.render(elem);
      elem.replaceWith(newHtml);
    }

    return raw.inject($.html(), raws);
  }
};
