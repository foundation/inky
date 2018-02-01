const htmlEqual = require('assert-html-equal');
const Inky = require('../../lib/inky');

/**
 * Takes HTML input, runs it through the Inky parser, and compares the output to what's expected.
 * @param {string} input - HTML input.
 * @param {string} expected - Expected HTML output.
 * @throws {Error} Throws an error if the output is not identical.
 */
module.exports = (input, expected, cheerioOpts) => {
  const inky = new Inky();
  const output = inky.releaseTheKraken(input, cheerioOpts);

  htmlEqual(output, expected);
};
