/* eslint-env mocha, browser */

const expect = require('chai').expect;

describe('Inky Browser', () => {
  it('exists on the window', () => {
    expect(window.Inky).to.be.a('function');
  });

  it('works correctly', () => {
    const inky = new window.Inky();
    const output = inky.releaseTheKraken('<Row></Row>');
    expect(output).to.contain('<table');
  });
});
