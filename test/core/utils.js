/* eslint-env mocha */

const expect = require('chai').expect;
const iff = require('../../lib/iff');

describe('iff()', () => {
  const string = 'hi';

  it('returns input string if condition is true', () => {
    expect(iff(true, string)).to.equal(string);
  });

  it('returns empty string if condition is false', () => {
    expect(iff(false, string)).to.equal('');
  });
});
