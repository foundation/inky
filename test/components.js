var compare = require('./lib/compare');

describe('Center', () => {
  it('applies a text-center class and center alignment attribute to the first child', () => {
    var input = `
      <center>
        <div></div>
      </center>
    `;
    var expected = `
      <center data-parsed="">
        <div align="center" class="float-center"></div>
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
        <center align="center" class="float-center" data-parsed="">
        </center>
      </center>
    `;

    compare(input, expected);
  });

  it('applies the class float-center to <item> elements', () => {
    var input = `
      <center>
        <menu>
          <item href="#"></item>
        </menu>
      </center>
    `;

    var expected = `
      <center data-parsed="">
        <table class="menu float-center" role="presentation" align="center">
          <tr>
            <td>
              <table role="presentation">
                <tr>
                  <th class="menu-item float-center">
                    <a href="#"></a>
                  </th>
                </tr>
              </table>
            </td>
          </tr>
        </table>
      </center>
    `;

    compare(input, expected);
  });
});

describe('Button', () => {
  it('creates a simple button', () => {
    var input = '<button href="http://zurb.com">Button</button>';
    var expected = `
      <table class="button" role="presentation">
        <tr>
          <td>
            <table role="presentation">
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
      <table class="button small alert" role="presentation">
        <tr>
          <td>
            <table role="presentation">
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
      <table class="button expand" role="presentation">
        <tr>
          <td>
            <table role="presentation">
              <tr>
                <td>
                  <center data-parsed=""><a href="http://zurb.com" align="center" class="float-center">Button</a></center>
                </td>
              </tr>
            </table>
          </td>
          <td class="expander"></td>
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
      <table class="menu" role="presentation">
        <tr>
          <td>
            <table role="presentation">
              <tr>
                <th class="menu-item"><a href="http://zurb.com">Item</a></th>
              </tr>
            </table>
          </td>
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
      <table class="menu vertical" role="presentation">
        <tr>
          <td>
            <table role="presentation">
              <tr>
              </tr>
            </table>
          </td>
        </tr>
      </table>
    `;

    compare(input, expected);
  });

  it('works without using an item tag', () => {
    var input = `
      <menu>
        <th class="menu-item"><a href="http://zurb.com">Item 1</a></th>
      </menu>
    `;
    var expected = `
      <table class="menu" role="presentation">
        <tr>
          <td>
            <table role="presentation">
              <tr>
                <th class="menu-item"><a href="http://zurb.com">Item 1</a></th>
              </tr>
            </table>
          </td>
        </tr>
      </table>
    `;

    compare(input, expected);
  });
});

describe('Callout', () => {
  it('creates a callout with correct syntax', () => {
    var input = '<callout>Callout</callout>';
    var expected = `
      <table class="callout" role="presentation">
        <tr>
          <th class="callout-inner">Callout</th>
          <th class="expander"></th>
        </tr>
      </table>
    `;

    compare(input, expected);
  });

  it('copies classes to the final HTML', () => {
    var input = '<callout class="primary">Callout</callout>';
    var expected = `
      <table class="callout" role="presentation">
        <tr>
          <th class="callout-inner primary">Callout</th>
          <th class="expander"></th>
        </tr>
      </table>
    `;

    compare(input, expected);
  });
});

describe('Spacer', () => {
  it('creates a spacer element with correct size', () => {
    var input = '<spacer size="10"></spacer>';
    var expected = `
      <table class="spacer" role="presentation">
        <tbody>
          <tr>
            <td height="10px" style="font-size:10px;line-height:10px;">&#xA0;</td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });

  it('copies classes to the final spacer HTML', () => {
    var input = '<spacer size="10" class="bgcolor"></spacer>';
    var expected = `
      <table class="spacer bgcolor" role="presentation">
        <tbody>
          <tr>
            <td height="10px" style="font-size:10px;line-height:10px;">&#xA0;</td>
          </tr>
        </tbody>
      </table>
    `;

    compare(input, expected);
  });
});

describe('wrapper', () => {
  it('creates a wrapper that you can attach classes to', () => {
    var input = `<wrapper class="header"></wrapper>`;
    var expected = `
      <table class="wrapper header" role="presentation" align="center">
        <tr>
          <td class="wrapper-inner"></td>
        </tr>
      </table>
    `;

    compare(input, expected);
  });
});
