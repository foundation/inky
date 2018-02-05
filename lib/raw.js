'use strict';

/**
 * These functions help with serializing and deserializing `<raw>` blocks within Inky HTML. Raw
 * blocks are ignored by the parser.
 */

module.exports = {
  /**
   * Find all uses of `<raw></raw>` and modify the input HTML to replace `<raw>...</raw>` with
   * `###RAW...###`.
   * @param {String} string - Input HTML.
   * @returns {Array} Array with list of extracted raw blocks at index 0, and modified input string at index 1.
   */
  extract(string) {
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
  },
  /**
   * Re-insert raw blocks extracted by `raw.extract()` into an input string.
   * @param {String} string - Input string.
   * @param {Object} raws - Raw blocks.
   * @returns {String} Modified string.
   */
  inject(string, raws) {
    let str = string;

    for (const i in raws) {
      if (Object.prototype.hasOwnProperty.call(raws, i)) {
        str = str.replace('###RAW' + i + '###', raws[i]);
      }
    }

    return str;
  }
};
