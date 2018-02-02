'use strict';

const extend = require('util')._extend;
const values = require('object-values');
const cheerio = require('cheerio');
const requireDir = require('require-dir');
const componentFactory = require('./component-factory');
const renderer = require('./renderer');

/**
 * Creates a new instance of the Inky parser.
 */
module.exports = class Inky {
  constructor(opts) {
    const options = opts || {};
    const components = requireDir('./components');

    this.cheerioOpts = options.cheerio;
    // HTML tags for custom components
    this.components = extend({
      callout: 'callout',
      spacer: 'spacer',
      wrapper: 'wrapper'
    }, options.components || {});

    // Column count for grid
    this.columnCount = options.columnCount || 12;

    this.library = new Map(
      Object.keys(components).map(k => [components[k].name, components[k]])
    );

    this.componentTags = values(this.components).concat(Array.from(this.library.keys()));

    this.render = renderer(this.library, this);
  }

  static mergeCheerioOpts(opts) {
    return Object.assign({
      decodeEntities: false,
      lowerCaseTags: false
    }, opts || {});
  }

  static extractRaws(string) {
    const raws = [];
    let i = 0;
    let raw;
    let str = string;
    const regex = /< *raw *>(.*?)<\/ *raw *>/i;

    while (raw = str.match(regex)) { // eslint-disable-line no-cond-assign
      raws[i] = raw[1];
      str = str.replace(regex, '###RAW' + i + '###');
      i += 1;
    }

    return [raws, str];
  }

  static reInjectRaws(string, raws) {
    let str = string;

    for (const i in raws) {
      if (Object.prototype.hasOwnProperty.call(raws, i)) {
        str = str.replace('###RAW' + i + '###', raws[i]);
      }
    }

    return str;
  }

  /**
   * Awww yiss. Kickstarts the whole parser. Takes in HTML as a string, checks if there are any custom components. If there are, it replaces the nested components, traverses the DOM and replaces them with email markup.
   * @param {object} $ - Input HTML as a string
   * @returns {object} Modified HTML as a string
   */
  releaseTheKraken(xmlString, cheerioOpts) {
    // This large compound selector looks for any custom tag loaded into Inky
    // <center> is an exception: the selector is center:not([data-parsed])
    // Otherwise the parser gets caught in an infinite loop where it continually tries to process the same <center> tags
    //
    // backwards compatible with old versions that pass in cheerio
    if (typeof (xmlString) !== 'string') {
      xmlString = xmlString.html();
    }

    const set = Inky.extractRaws(xmlString);
    const raws = set[0];
    let string = set[1];
    const $ = cheerio.load(string, Inky.mergeCheerioOpts(cheerioOpts));
    const tags = this.componentTags.map(tag => `${tag}:not([data-parsed])`).join(', ');

    // Because the structure of the DOM constantly shifts, we carefully go through each custom tag one at a time, until there are no more custom tags to parse
    while ($(tags).length > 0) {
      const elem = $(tags).eq(0);
      const newHtml = this.componentFactory(elem);
      elem.replaceWith(newHtml);
    }

    // Remove data-parsed attributes created for <center>
    $('[data-parsed]').removeAttr('data-parsed');

    string = $.html();
    return Inky.reInjectRaws(string, raws);
  }

  componentFactory() {
    return componentFactory.apply(this, arguments);
  }
};
