'use strict';

const cheerio = require('cheerio');
const deepAssign = require('deep-assign');
const renderer = require('./renderer');
const raw = require('./raw');
const components = require('./components');

/**
 * Creates a new instance of the Inky parser.
 */
module.exports = class Inky {
  /**
   * Create a new instance of the Inky parser.
   * @param {Object} opts - Parser options.
   */
  constructor(opts) {
    this.options = deepAssign({
      cheerio: {
        decodeEntities: false,
        lowerCaseTags: false
      },
      columnCount: 12,
      components: []
    }, opts || {});

    this.library = new Map(
      components.concat(this.options.components).map(component => [component.name, component])
    );

    this.selectors = Array.from(this.library.keys()).join(', ');

    this.render = renderer(this.library, this.options);
  }

  /**
   * Awww yiss. Kickstarts the whole parser. Takes in HTML as a string, checks if there are any custom components. If there are, it replaces the nested components, traverses the DOM and replaces them with email markup.
   * @param {String} input - Inky HTML.
   * @returns {String} Converted HTML.
   */
  releaseTheKraken(input) {
    const set = raw.extract(input);
    const raws = set[0];
    const string = set[1];
    const $ = cheerio.load(string, this.options.cheerio);

    // Because the structure of the DOM constantly shifts, we carefully go through each custom tag one at a time, until there are no more custom tags to parse
    while ($(this.selectors).length > 0) {
      const elem = $(this.selectors).eq(0);
      const newHtml = this.render(elem);
      elem.replaceWith(newHtml);
    }

    return raw.inject($.html(), raws);
  }
};
