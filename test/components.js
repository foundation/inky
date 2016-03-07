var compare = require('./lib/compare');

describe('Center', () => {
  it('applies a center class and center alignment attribute to the first child', () => {
    var input = `
      <center>
        <div></div>
      </center>
    `;
    var expected = `
      <center data-parsed="">
        <div align="center" class="center"></div>
      </center>
    `;

    compare(input, expected);
  });

  it(`doesn't choke if center tags are nested`, () => {
    var input = `
      <center>
        <center>
        </center>
      </center>
    `;

    var expected = `
      <center data-parsed="">
        <center align="center" class="center" data-parsed="">
        </center>
      </center>
    `;

    compare(input, expected);
  });
});

describe('Button', () => {
  it('creates a simple button', () => {
    var input = '<button href="http://zurb.com">Button</button>';
    var expected = `
      <table class="button">
        <tr>
          <td>
            <table>
              <tr>
                <td><a href="http://zurb.com">Button</a></td>
              </tr>
            </table>
          </td>
        </tr>
      </table>
    `;

    compare(input, expected);
  });

  it('creates a button with classes', () => {
    var input = `
      <button class="small alert" href="http://zurb.com">Button</button>
    `;
    var expected = `
      <table class="button small alert">
        <tr>
          <td>
            <table>
              <tr>
                <td><a href="http://zurb.com">Button</a></td>
              </tr>
            </table>
          </td>
        </tr>
      </table>
    `;

    compare(input, expected);
  });

  it('creates a correct expanded button', () => {
    var input = `
      <button class="expand" href="http://zurb.com">Button</button>
    `;
    var expected = `
      <table class="button expand">
        <tr>
          <td>
            <table>
              <tr>
                <td>
                  <center data-parsed=""><a href="http://zurb.com" align="center" class="center">Button</a></center>
                </td>
              </tr>
            </table>
          </td>
        </tr>
      </table>
    `;

    compare(input, expected);
  });
});

describe('Menu', () => {
  it('creates a menu with item tags inside', () => {
    var input = `
      <menu>
        <item href="http://zurb.com">Item</item>
      </menu>
    `;
    var expected = `
      <table class="menu">
        <tr>
          <td><a href="http://zurb.com">Item</a></td>
        </tr>
      </table>
    `;

    compare(input, expected);
  });

  it('creates a menu with classes', () => {
    var input = `
      <menu class="vertical">
      </menu>
    `;
    var expected = `
      <table class="menu vertical">
        <tr>
        </tr>
      </table>
    `;

    compare(input, expected);
  });

  it('works without using an item tag', () => {
    var input = `
      <menu>
        <td><a href="http://zurb.com">Item 1</a></td>
        <td><a href="http://zurb.com">Item 2</a></td>
      </menu>
    `;
    var expected = `
      <table class="menu">
        <tr>
          <td><a href="http://zurb.com">Item 1</a></td>
          <td><a href="http://zurb.com">Item 2</a></td>
        </tr>
      </table>
    `;

    compare(input, expected);
  });
});

describe('Callout', () => {
  it('creates a callout with correct syntax', () => {
    var input = '<callout></callout>';
    var expected = `
      <table>
        <tr>
          <td class="callout"></td>
        </tr>
      </table>
    `;

    compare(input, expected);
  });

  it('copies classes to the final HTML', () => {
    var input = '<callout class="primary"></callout>';
    var expected = `
      <table>
        <tr>
          <td class="callout primary"></td>
        </tr>
      </table>
    `;

    compare(input, expected);
  });
});
