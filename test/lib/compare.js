const htmlEqual = require('assert-html-equal');
const Inky = require('../../lib/inky');

/**
 * Takes HTML input, runs it through the Inky parser, and compares the output to what's expected.
 * @param {String} input - HTML input.
 * @param {String} expected - Expected HTML output.
 * @param {Object} [options] - Inky options.
 * @throws {Error} Throws an error if the output is not identical.
 */
module.exports = (input, expected, options) => {
  const inky = new Inky(options);
  const output = inky.releaseTheKraken(input);

  htmlEqual(output, expected);
};
